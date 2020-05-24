use grr::ShaderStage;
use slotmap::{DenseSlotMap, new_key_type};
use std::borrow::ToOwned;
use std::cell::Cell;
// use std::collections::HashSet;
use std::path::{Path, PathBuf};
use itertools::process_results;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShaderSource {
    SourceFile(PathBuf),
    Literal(String),
}

new_key_type! {
    pub struct ManagedShader;
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
    pub fn from_source<T: AsRef<Path>>(source_path: T, stage: ShaderStage) -> ShaderDesc {
        ShaderDesc {
            source: ShaderSource::SourceFile(source_path.as_ref().to_owned()),
            stage
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
    fn is_compatible(&self, s: ShaderStage) -> bool {
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

#[derive(Clone)]
struct Pipeline {
    pipeline: Cell<grr::Pipeline>,
    pipeline_type: PipelineType,
    shaders: Vec<ShaderDesc>,
}

#[derive(Debug, Clone)]
pub enum Error {
    GrrError(grr::Error),
    CompilationError(ShaderSource, String),
    NoShadersToLink,
    UncompiledShader,
    IncompatibleShaderTypes,
    MissingPipeline,
    LinkError(String),
    FileError(PathBuf),
}

/// The shader manager keeps track of all shader objects and
/// pipelines, and managing the relationship between them.
pub struct ShaderManager {
    //shader_descs: HashSet<ShaderDesc>,
    pipelines: DenseSlotMap<ManagedPipeline, Pipeline>,
}

impl<'a> ShaderManager {
    pub fn new() -> ShaderManager {
        ShaderManager {
            pipelines: DenseSlotMap::with_key(),
        }
    }

    /// Attempt to compile a shader from a `ShaderSource`.
    ///
    /// Returns the created shader if it compiled successfully.
    fn load_shader(&self, device: &grr::Device, desc: &ShaderDesc) -> Result<grr::Shader, Error> {
        let s = match desc.source.clone() {
            ShaderSource::SourceFile(path) => {
                let b = std::fs::read_to_string(&path).map_err(|_| Error::FileError(path))?;
                b
            }
            ShaderSource::Literal(s) => s,
        };

        let shader = unsafe {
            device
                .compile_shader(desc.stage, s.as_bytes())
                .map_err(|e| Error::GrrError(e))?
        };

        let shader_log = unsafe { device.get_shader_compile_log(shader) };
        match shader_log {
            Ok(_) => Ok(shader),
            Err(error_log) => {
                unsafe {
                    device.delete_shader(shader);
                }
                println!("{}", error_log);
                Err(Error::CompilationError(desc.source.clone(), error_log))
            }
        }
    }
    /// Return a raw pipeline if all of the shaders compile and all of
    /// the links are successful.
    fn load_pipeline(&self,
                     device: &grr::Device,
                     shaders: &[ShaderDesc],
                     ptype: Option<PipelineType>,
    ) -> Result<(grr::Pipeline, PipelineType), Error> {
        if shaders.len() == 0 {
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
            .map(|s| self.load_shader(device, &s))
            .collect();

        let raw_shaders: Vec<_> = process_results(raw_shaders, |iter| iter.collect())?;

        let pipeline = unsafe {
            device
                .create_pipeline(&raw_shaders)
                .map_err(|x| Error::GrrError(x))?
        };

        // delete all of the shaders
        raw_shaders.iter().for_each(|s| {
            unsafe {
                device.delete_shader(*s);
            }
        });

        let plog = unsafe { device.get_pipeline_log(pipeline) };
        match plog {
            Ok(_) => {
                Ok((pipeline, pipeline_type))
            }
            Err(e) => {
                unsafe {
                    device.delete_pipeline(pipeline);
                }
                Err(Error::LinkError(e))
            }
        }

    }
    /// Create and link a program
    pub fn create_pipeline(
        &mut self,
        device: &grr::Device,
        shaders: &[ShaderDesc],
        ptype: Option<PipelineType>,
    ) -> Result<ManagedPipeline, Error> {
        self.load_pipeline(device, shaders, ptype).map(|(p, pipeline_type)| {
            self.pipelines.insert(Pipeline {
                shaders: shaders.iter().cloned().collect(),
                pipeline: Cell::new(p),
                pipeline_type
            })
        })
    }

    /// Reload all of the shaders associated with the pipeline, and
    /// relink the pipeline. If any of the steps fail, the underlying
    /// program does not change at all.
    pub fn reload_all_pipelines(&self, device: &grr::Device) {
        // Try to re-create every pipeline
        for pipeline in self.pipelines.values() {
            let new_pipeline_raw = self.load_pipeline(device, &pipeline.shaders, Some(pipeline.pipeline_type));
            if let Ok((new_p, _)) = new_pipeline_raw {
                pipeline.pipeline.replace(new_p);
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
        descs
            .iter()
            .find(|&x| !ptype.is_compatible(x.stage))
            .is_some()
    }

    /// return a handle to the raw grr::Pipeline
    pub fn pipeline_handle(&self, pipeline: ManagedPipeline) -> Option<grr::Pipeline> {
        self.pipelines.get(pipeline).map(|s| s.pipeline.get())
    }

    fn map_pipeline<T, F: Fn(grr::Pipeline) -> T>(&self, pipeline: ManagedPipeline, f: F) -> Result<T, Error> {
        match self.pipelines.get(pipeline) {
            Some(p) => {
                Ok(f(p.pipeline.get()))
            },
            None => {
                Err(Error::MissingPipeline)
            }
        }
    }

    /// Bind the pipeline.
    pub fn bind_pipeline(&self, device: &grr::Device, pipeline: ManagedPipeline) -> Result<(), Error> {
        self.map_pipeline(pipeline, |p| unsafe { device.bind_pipeline(p); })
    }

    /// Delete teh pipeline.
    pub fn delete_pipeline(&self, device: &grr::Device, pipeline: ManagedPipeline) -> Result<(), Error> {
        self.map_pipeline(pipeline, |p| unsafe { device.delete_pipeline(p); })
    }
}
