//! Manager for images, image views, and samplers
use crate::image_format::*;
use slotmap::{new_key_type, DenseSlotMap};
use thiserror::Error;

new_key_type! {
    pub struct ImageId;
}

new_key_type! {
    pub struct ImageViewId;
}

/// Internal structure information for an image.
struct Image {
    handle: grr::Image,
    image_type: grr::ImageType,
    num_mipmap_levels: u32,
    format: grr::Format,
}

impl Image {
    pub fn handle(&self) -> grr::Image {
        self.handle
    }
}

/// Internal structure information for texture that specifically
/// represents an image view.
struct ImageView {
    handle: grr::ImageView,

    #[allow(unused)]
    orig_handle: ImageId,

    #[allow(unused)]
    image_view_type: grr::ImageViewType,
    #[allow(unused)]
    num_layers: u32,
    #[allow(unused)]
    num_mipmap_levels: u32,
    #[allow(unused)]
    format: grr::Format,
}

impl ImageView {
    pub fn handle(&self) -> grr::ImageView {
        self.handle
    }
}

/// Errors from `ImageManager`
#[derive(Debug, Error)]
pub enum Error {
    /// Internal `grr` error
    GrrError(#[from] grr::Error),
    MissingImageId(ImageId),
    BadDataLayout,
    ImproperDataFormat,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::GrrError(e) => write!(f, "GrrError: {}", e),
            Error::MissingImageId(_) => write!(f, "MisisngImageId"),
            Error::BadDataLayout => write!(f, "BadDataLayout"),
            Error::ImproperDataFormat => write!(f, "ImproperDataFormat"),
        }
    }
}

pub enum ImageOrViewId {
    Image(ImageId),
    View(ImageViewId),
}

impl From<ImageViewId> for ImageOrViewId {
    fn from(view_id: ImageViewId) -> ImageOrViewId {
        ImageOrViewId::View(view_id)
    }
}
impl From<ImageId> for ImageOrViewId {
    fn from(image_id: ImageId) -> ImageOrViewId {
        ImageOrViewId::Image(image_id)
    }
}

/// Create and bind images, with caching for image properties.
pub struct ImageManager<'d> {
    images: DenseSlotMap<ImageId, Image>,
    views: DenseSlotMap<ImageViewId, ImageView>,
    device: &'d grr::Device,
}

impl<'d> ImageManager<'d> {
    pub fn new(device: &'d grr::Device) -> ImageManager {
        ImageManager {
            images: DenseSlotMap::with_key(),
            views: DenseSlotMap::with_key(),
            device,
        }
    }

    /// Create a new image with the specified storage format and type.
    pub fn create_image(
        &mut self,
        image_type: grr::ImageType,
        format: grr::Format,
        num_mipmap_levels: u32,
    ) -> Result<ImageId, Error> {
        let handle = unsafe {
            self.device
                .create_image(image_type, format, num_mipmap_levels)?
        };

        Ok(self.images.insert(Image {
            handle,
            image_type,
            format,
            num_mipmap_levels,
        }))
    }

    pub fn create_image_from_ndarray<PC: TexelBaseType, D: TextureDim, CD: TextureComponentDim>(
        &mut self,
        data: &ndarray::Array<nalgebra::VectorN<PC, CD>, D>,
        num_mip_map_levels: u32,
        gen_mipmaps: bool,
    ) -> Result<ImageId, Error>
    where
        nalgebra::DefaultAllocator: nalgebra::base::allocator::Allocator<PC, CD>,
    {
        let d = data.as_slice().ok_or(Error::ImproperDataFormat)?;

        let image_type = data.raw_dim().image_type();
        let base_format = CD::base_format;
        let format_layout = PC::layout;
        let format = format_from_base_and_layout(base_format, format_layout)
            .ok_or(Error::ImproperDataFormat)?;

        let handle = self.create_image(image_type, format, num_mip_map_levels)?;

        // copy the image data to the client
        let sub_level = grr::SubresourceLayers {
            level: 0,
            layers: 0..1,
        };
        let sub_layout = grr::MemoryLayout {
            base_format,
            format_layout,
            row_length: 0,
            image_height: 0,
            alignment: 1,
        };

        let image = self.images[handle].handle();
        unsafe {
            self.device.copy_host_to_image(
                d,
                image,
                grr::HostImageCopy {
                    host_layout: sub_layout,
                    image_extent: image_type.full_extent(),
                    image_offset: grr::Offset { x: 0, y: 0, z: 0 },
                    image_subresource: sub_level,
                },
            );
        }

        if gen_mipmaps {
            unsafe {
                self.device.generate_mipmaps(image);
            }
        };

        Ok(handle)
    }

    /// Create a new image view, using the full image.
    pub fn create_image_view_whole(&mut self, image_id: ImageId) -> Result<ImageViewId, Error> {
        let image = match self.images.get(image_id) {
            Some(img) => img,
            None => {
                return Err(Error::MissingImageId(image_id));
            }
        };

        let image_view_type = image_type_to_view_type(image.image_type);
        let num_layers = match image.image_type {
            grr::ImageType::D1 { layers, .. } => layers,
            grr::ImageType::D2 { layers, .. } => layers,
            grr::ImageType::D3 { .. } => 1,
        };

        let sub_range = grr::SubresourceRange {
            levels: 0..image.num_mipmap_levels,
            layers: 0..num_layers,
        };

        let handle = unsafe {
            self.device
                .create_image_view(image.handle, image_view_type, image.format, sub_range)?
        };

        Ok(self.views.insert(ImageView {
            handle,
            orig_handle: image_id,
            image_view_type,
            num_layers,
            num_mipmap_levels: image.num_mipmap_levels,
            format: image.format,
        }))
    }

    /// Return the texture as a packed vector.
    pub fn get_texture_vec<T: TexelBaseType>(&self, image_id: ImageId) -> Result<Vec<T>, Error> {
        let image = self
            .images
            .get(image_id)
            .ok_or(Error::MissingImageId(image_id))?;

        let num_texels = image.image_type.num_texels() * image.format.num_components() as usize;
        let mut texture_data: Vec<T> = Vec::with_capacity(num_texels);
        texture_data.resize(num_texels, T::zero());

        let mem_layout = grr::MemoryLayout {
            base_format: image.format.base_format(),
            format_layout: T::layout,
            row_length: 0,
            image_height: 0,
            alignment: 1,
        };

        unsafe {
            self.device.copy_image_to_host(
                image.handle(),
                &mut texture_data,
                grr::HostImageCopy {
                    host_layout: mem_layout,
                    image_subresource: grr::SubresourceLayers {
                        level: 0,
                        layers: 0..1,
                    },
                    image_offset: grr::Offset::ORIGIN,
                    image_extent: image.image_type.full_extent(),
                },
            );
        }

        Ok(texture_data)
    }

    pub fn get_image_handle(&self, image: ImageId) -> Option<grr::Image> {
        self.images.get(image).map(|x| x.handle())
    }

    pub fn get_image_view_handle(&self, view: ImageViewId) -> Option<grr::ImageView> {
        self.views.get(view).map(|x| x.handle())
    }

    /// Delete an existing image.
    pub fn delete_image(&mut self, image: ImageId) {
        if let Some(img) = self.images.remove(image) {
            unsafe {
                self.device.delete_image(img.handle);
            }
        }
    }

    /// Delete an image view.
    pub fn delete_image_view(&mut self,view: ImageViewId) {
        if let Some(v) = self.views.remove(view) {
            unsafe {
                self.device.delete_image_view(v.handle);
            }
        }
    }

    /// Assign a name to the internal OpenGL object represented by the
    /// image or view.
    ///
    /// Useful for debugging (especially with RenderDoc).
    pub fn assign_label(&self, img: impl Into<ImageOrViewId>, label: &str) {
	match img.into() {
	    ImageOrViewId::Image(i) => {
		self.get_image_handle(i).map(|handle| {
		    unsafe {
			self.device.object_name(handle, label);
		    }
		});
	    },
	    ImageOrViewId::View(v) => {
		self.get_image_view_handle(v).map(|handle| {
		    unsafe {
			self.device.object_name(handle, label);
		    }
		});
	    }
	}
    }

    /// Delete all images and views.
    pub fn clear(&mut self) {
        for (_id, view) in self.views.drain() {
            unsafe {
                self.device.delete_image_view(view.handle);
            }
        }
        for (_id, image) in self.images.drain() {
            unsafe {
                self.device.delete_image(image.handle);
            }
        }
    }

    /// Bind the image view to image storage.
    pub fn bind_storage(&self, bind_point: u32, view: ImageViewId) {
        unsafe {
            if let Some(v) = self.views.get(view) {
                self.device
                    .bind_storage_image_views(bind_point, &[v.handle]);
            }
        }
    }

    /// Bind the image view to a sampler.
    pub fn bind(&self, bind_point: u32, view: ImageViewId) {
        unsafe {
            if let Some(v) = self.views.get(view) {
                self.device.bind_image_views(bind_point, &[v.handle]);
            }
        }
    }
}
