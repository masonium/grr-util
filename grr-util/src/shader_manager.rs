use grr::{PipelineFlags, ShaderFlags, ShaderStage};
use slotmap::{new_key_type, DenseSlotMap};
use std::borrow::ToOwned;
use std::cell::Cell;
// use std::collections::HashSet;
use itertools::process_results;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Representation of where shader description can come from.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShaderSource {
    /// GLSL filename
    SourceFile(PathBuf),

    /// String literal for hard-coded or in-program data.
    Literal(String),
}

impl std::fmt::Display for ShaderSource {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::SourceFile(p) => write!(fmt, "SourcePath({:?})", p),
            Self::Literal(s) => {
                let st: &str = &s;
                write!(
                    fmt,
                    "Literal(lines={})",
                    st.chars().take(100).collect::<String>()
                )
            }
        }
    }
}

new_key_type! {
    /// `ManagedPipeline` is a key to reference a program.
    ///
    /// `ManagedPipeline`s are guaranteed to represent to a valid
    /// pipeline once loaded.
    pub struct ManagedPipeline;
}

#[derive(Clone, PartialEq, Eq, Hash)]
/// `Shader` represents the information necessary to compile (or
/// recompile) a shader and link it to a pipeline.
pub struct ShaderDesc {
    source: ShaderSource,
    stage: ShaderStage,
}

impl ShaderDesc {
    pub fn from_file<T: AsRef<Path>>(source_path: T, stage: ShaderStage) -> ShaderDesc {
        ShaderDesc {
            source: ShaderSource::SourceFile(source_path.as_ref().to_owned()),
            stage,
        }
    }
    pub fn from_raw(source: String, stage: ShaderStage) -> ShaderDesc {
        ShaderDesc {
            source: ShaderSource::Literal(source),
            stage,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PipelineType {
    Graphics,
    Compute,
    Mesh,
}

impl PipelineType {
    fn is_compatible(self, s: ShaderStage) -> bool {
        use ShaderStage::*;
        match &self {
            PipelineType::Graphics => match s {
                Vertex | TessellationControl | TessellationEvaluation | Geometry | Fragment => true,
                _ => false,
            },
            PipelineType::Compute => match s {
                Compute => true,
                _ => false,
            },
            PipelineType::Mesh => match s {
                MeshNv | TaskNv | Fragment => true,
                _ => false,
            },
        }
    }
}

/// Based on the name of the shader filename, guess the
/// `grr::ShaderStage` of the shader.
fn guess_stage<P: AsRef<Path>>(filename: P) -> Result<grr::ShaderStage, Error> {
    let path: &Path = filename.as_ref();
    let path_string = path.to_string_lossy();
    // strip the glsl part from the ending, if it exists.
    let path_string = path_string.trim_end_matches(".glsl");

    if path_string.ends_with(".vert") {
        Ok(grr::ShaderStage::Vertex)
    } else if path_string.ends_with(".frag") {
        Ok(grr::ShaderStage::Fragment)
    } else if path_string.ends_with(".comp") {
        Ok(grr::ShaderStage::Compute)
    } else if path_string.ends_with(".geom") {
        Ok(grr::ShaderStage::Geometry)
    } else if path_string.ends_with(".tesc") {
        Ok(grr::ShaderStage::TessellationControl)
    } else if path_string.ends_with(".tese") {
        Ok(grr::ShaderStage::TessellationEvaluation)
    } else if path_string.ends_with(".mesh") {
        Ok(grr::ShaderStage::MeshNv)
    } else if path_string.ends_with(".task") {
        Ok(grr::ShaderStage::TaskNv)
    } else {
        Err(Error::UnknownStage(path.to_owned()))
    }
}

#[derive(Clone)]
struct Pipeline {
    pipeline: Cell<grr::Pipeline>,
    pipeline_type: PipelineType,
    shaders: Vec<ShaderDesc>,

    /// Internal name for underlying grr::Pipeline (e.g. OpenGL program).
    base_name: Option<String>,

    /// Reload count, for naming pipelines when provided.
    iteration: usize,
}

impl Pipeline {
    fn name(&self) -> Option<String> {
	self.base_name.as_ref().map(|s| format!("{} RL{}", s, self.iteration))
    }
}

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("internal grr::Error")]
    GrrError(#[from] grr::Error),

    #[error("failed to compiler shader source {0}: {1}")]
    CompilationError(ShaderSource, String),

    #[error("empty shader list")]
    NoShadersToLink,

    #[error("trying to link uncompiled shader")]
    UncompiledShader,

    #[error("linking incompatible shader")]
    IncompatibleShaderTypes,

    #[error("Could not guess the stage from filename {0}")]
    UnknownStage(PathBuf),

    #[error("Cannot find pipeline")]
    MissingPipeline,

    #[error("failed to link pipelines: {0}")]
    LinkError(String),

    #[error("internal file error from {0}")]
    FileError(PathBuf),
}

/// The shader manager keeps track of all shader objects and
/// pipelines, and managing the relationship between them.
pub struct ShaderManager<'d> {
    //shader_descs: HashSet<ShaderDesc>,
    device: &'d grr::Device,
    pipelines: DenseSlotMap<ManagedPipeline, Pipeline>,
}

impl<'d> ShaderManager<'d> {
    pub fn new(device: &'d grr::Device) -> ShaderManager {
        ShaderManager {
            pipelines: DenseSlotMap::with_key(),
            device,
        }
    }

    /// Attempt to compile a shader from a `ShaderSource`.
    ///
    /// Returns the created shader if it compiled successfully.
    fn load_shader(&self, desc: &ShaderDesc) -> Result<grr::Shader, Error> {
        let s = match desc.source.clone() {
            ShaderSource::SourceFile(path) => {
                std::fs::read_to_string(&path).map_err(|_| Error::FileError(path))?
            }
            ShaderSource::Literal(s) => s,
        };

        let shader = unsafe {
            self.device.create_shader(
                desc.stage,
                grr::ShaderSource::Glsl,
                s.as_bytes(),
                ShaderFlags::empty(),
            )
        };

        match shader {
            Ok(s) => Ok(s),
            Err(grr::Error::CompileError(s)) => {
                let shader_log = unsafe { self.device.get_shader_log(s) };
                unsafe {
                    self.device.delete_shader(s);
                }
                Err(Error::CompilationError(
                    desc.source.clone(),
                    shader_log.unwrap_or_default(),
                ))
            }
            Err(e) => Err(Error::GrrError(e)),
        }
    }

    /// Return a raw pipeline if all of the shaders compile and all of
    /// the links are successful.
    fn load_pipeline(
        &self,
        shaders: &[ShaderDesc],
        ptype: Option<PipelineType>,
    ) -> Result<(grr::Pipeline, PipelineType), Error> {
        if shaders.is_empty() {
            return Err(Error::NoShadersToLink);
        }

        let pipeline_type = match ptype {
            Some(x) => {
                if self.verify_pipeline_type(shaders, x) {
                    x
                } else {
                    return Err(Error::IncompatibleShaderTypes);
                }
            }
            None => match self.derive_pipeline_type(shaders) {
                Some(p) => p,
                None => {
                    return Err(Error::IncompatibleShaderTypes);
                }
            },
        };

        let raw_shaders: Vec<_> = shaders
            .iter()
            .map(|s| self.load_shader(&s))
            .collect();

        let raw_shaders: Vec<_> = process_results(raw_shaders, |iter| iter.collect())?;

        let pipeline = unsafe {
            self.device
                .create_pipeline(&raw_shaders, PipelineFlags::empty())
        };

        // delete all of the shaders
        raw_shaders.iter().for_each(|s| unsafe {
            self.device.delete_shader(*s);
        });

        match pipeline {
            Ok(p) => Ok((p, pipeline_type)),
            Err(grr::Error::LinkError(p)) => {
                let plog = unsafe { self.device.get_pipeline_log(p) };
                unsafe {
                    self.device.delete_pipeline(p);
                }
                Err(Error::LinkError(plog.unwrap_or_default()))
            }
            Err(e) => Err(Error::GrrError(e)),
        }
    }

    /// Create and link a program
    pub fn create_pipeline(
        &mut self,
        shaders: &[ShaderDesc],
        ptype: Option<PipelineType>,
    ) -> Result<ManagedPipeline, Error> {
        self.load_pipeline(shaders, ptype)
            .map(|(p, pipeline_type)| {
                self.pipelines.insert(Pipeline {
                    shaders: shaders.to_vec(),
                    pipeline: Cell::new(p),
                    pipeline_type,
		    base_name: None,
		    iteration: 0
                })
            })
    }

    /// Create and link a program from file shaders.
    pub fn create_pipeline_from_files<P: AsRef<Path>>(
        &mut self,
        shader_filenames: &[P],
    ) -> Result<ManagedPipeline, Error> {
        let mut shader_descs = vec![];
        for filename in shader_filenames {
            shader_descs.push(ShaderDesc::from_file(filename, guess_stage(filename)?));
        }

        self.create_pipeline(&shader_descs, None)
    }

    /// Reload all of the shaders associated with the pipeline, and
    /// relink the pipeline. If any of the steps fail, the underlying
    /// program does not change at all.
    pub fn reload_all_pipelines(&self) {
        // Try to re-create every pipeline
        for pipeline in self.pipelines.values() {
            let new_pipeline_raw =
                self.load_pipeline(&pipeline.shaders, Some(pipeline.pipeline_type));
            if let Ok((new_p, _)) = new_pipeline_raw {
                pipeline.pipeline.replace(new_p);
		if let Some(name) = pipeline.name() {
		    unsafe {
			self.device.object_name(new_p, &name);
		    }
		}
            }
        }
    }

    /// Use the types of the shaders to derive the type of the pipeline.
    fn derive_pipeline_type(&self, shaders: &[ShaderDesc]) -> Option<PipelineType> {
        [
            PipelineType::Graphics,
            PipelineType::Compute,
            PipelineType::Mesh,
        ]
        .iter()
        .find(|&&ptype| self.verify_pipeline_type(shaders, ptype))
        .cloned()
    }

    /// Verify that the pipleine type matches the shader list.
    /// Assumes that there is at least one shader in the list.
    fn verify_pipeline_type(&self, descs: &[ShaderDesc], ptype: PipelineType) -> bool {
        descs.iter().any(|x| !ptype.is_compatible(x.stage))
    }

    /// Return a handle to the raw grr::Pipeline
    pub fn pipeline_handle(&self, pipeline: ManagedPipeline) -> Option<grr::Pipeline> {
        self.pipelines.get(pipeline).map(|s| s.pipeline.get())
    }

    fn map_pipeline<T, F: Fn(grr::Pipeline) -> T>(
        &self,
        pipeline: ManagedPipeline,
        f: F,
    ) -> Result<T, Error> {
        match self.pipelines.get(pipeline) {
            Some(p) => Ok(f(p.pipeline.get())),
            None => Err(Error::MissingPipeline),
        }
    }

    /// Assign a name to the underlying OpenGL Program object, for debugging purposes.
    ///
    /// Names of reloaded pipelines are preserved, modulo a -RL# tag
    /// to indicate the reload iteration count.
    pub fn assign_label(&mut self, pipeline: ManagedPipeline, label: &str) {
        if let Some(p) = self.pipelines.get_mut(pipeline) {
	    p.base_name = Some(label.to_string());
	    unsafe {
		self.device.object_name(p.pipeline.get(), &p.name().unwrap());
	    }
	}
    }
    /// Bind the pipeline.
    pub fn bind_pipeline(&self, pipeline: ManagedPipeline) -> Result<(), Error> {
        self.map_pipeline(pipeline, |p| unsafe {
            self.device.bind_pipeline(p);
        })
    }

    /// Bind uniforms to the pipeline.
    pub fn bind_uniform_constants(
        &self,
        pipeline: ManagedPipeline,
        first: u32,
        constants: &[grr::Constant],
    ) -> Result<(), Error> {
        self.map_pipeline(pipeline, |p| unsafe {
            self.device.bind_uniform_constants(p, first, constants)
        })
    }

    /// Delete the pipeline.
    pub fn delete_pipeline(&self, pipeline: ManagedPipeline) -> Result<(), Error> {
        self.map_pipeline(pipeline, |p| unsafe {
            self.device.delete_pipeline(p);
        })
    }

    /// Delete all pipelines managed by this manager.
    pub fn clear(&mut self) {
        for (_, p) in self.pipelines.drain() {
            unsafe {
                self.device.delete_pipeline(p.pipeline.into_inner());
            }
        }
    }
}
