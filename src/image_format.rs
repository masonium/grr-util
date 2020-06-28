#![allow(non_upper_case_globals)]

use grr::FormatLayout;
use num_traits::Zero;

pub trait TexelBaseType: Zero + PartialEq + Clone + Copy + std::fmt::Debug + 'static {
    const layout: FormatLayout;
}

// ndarray dimensions that can be interpreted as a texture dimension.
pub trait TextureDim: ndarray::Dimension {
    fn image_type(&self) -> grr::ImageType;
}

impl TextureDim for ndarray::Ix1 {
    fn image_type(&self) -> grr::ImageType {
        grr::ImageType::D1 {
            width: self[0] as u32,
            layers: 1,
        }
    }
}

impl TextureDim for ndarray::Ix2 {
    fn image_type(&self) -> grr::ImageType {
        grr::ImageType::D2 {
            width: self[1] as u32,
            height: self[0] as u32,
            layers: 1,
            samples: 1,
        }
    }
}

impl TextureDim for ndarray::Ix3 {
    fn image_type(&self) -> grr::ImageType {
        grr::ImageType::D3 {
            width: self[2] as u32,
            height: self[1] as u32,
            depth: self[2] as u32,
        }
    }
}

pub trait TextureComponentDim: nalgebra::Dim + nalgebra::DimName {
    const base_format: grr::BaseFormat;
}

impl TextureComponentDim for nalgebra::base::dimension::U1 {
    const base_format: grr::BaseFormat = grr::BaseFormat::R;
}
impl TextureComponentDim for nalgebra::base::dimension::U2 {
    const base_format: grr::BaseFormat = grr::BaseFormat::RG;
}
impl TextureComponentDim for nalgebra::base::dimension::U3 {
    const base_format: grr::BaseFormat = grr::BaseFormat::RGB;
}
impl TextureComponentDim for nalgebra::base::dimension::U4 {
    const base_format: grr::BaseFormat = grr::BaseFormat::RGBA;
}

impl TexelBaseType for f32 {
    const layout: grr::FormatLayout = grr::FormatLayout::F32;
}
impl TexelBaseType for u8 {
    const layout: grr::FormatLayout = grr::FormatLayout::U8;
}
impl TexelBaseType for u16 {
    const layout: grr::FormatLayout = grr::FormatLayout::U16;
}
impl TexelBaseType for u32 {
    const layout: grr::FormatLayout = grr::FormatLayout::U32;
}
impl TexelBaseType for i8 {
    const layout: grr::FormatLayout = grr::FormatLayout::I8;
}
impl TexelBaseType for i16 {
    const layout: grr::FormatLayout = grr::FormatLayout::I16;
}
impl TexelBaseType for i32 {
    const layout: grr::FormatLayout = grr::FormatLayout::I32;
}

/// Return a full format from a base format and a format layout.
pub fn format_from_base_and_layout(
    bf: grr::BaseFormat,
    layout: grr::FormatLayout,
) -> Option<grr::Format> {
    match (bf, layout) {
        (grr::BaseFormat::R, grr::FormatLayout::F32) => Some(grr::Format::R32_SFLOAT),
        (grr::BaseFormat::RG, grr::FormatLayout::F32) => Some(grr::Format::R32G32_SFLOAT),
        (grr::BaseFormat::RGB, grr::FormatLayout::F32) => Some(grr::Format::R32G32B32_SFLOAT),
        (grr::BaseFormat::RGBA, grr::FormatLayout::F32) => Some(grr::Format::R32G32B32A32_SFLOAT),
        _ => None,
    }
}

/// Return the image view type most closely matching the image type.
pub fn image_type_to_view_type(img_type: grr::ImageType) -> grr::ImageViewType {
    match img_type {
        grr::ImageType::D1 { layers, .. } if layers == 1 => grr::ImageViewType::D1,
        grr::ImageType::D1 { .. } => grr::ImageViewType::D1Array,
        grr::ImageType::D2 { layers, .. } if layers == 1 => grr::ImageViewType::D2,
        grr::ImageType::D2 { .. } => grr::ImageViewType::D2Array,
        grr::ImageType::D3 { .. } => grr::ImageViewType::D3,
    }
}

/// Given an `ImageType` return an `Extent` object that fully encompasses it.
pub fn image_type_to_full_extent(img_type: grr::ImageType) -> grr::Extent {
    grr::Extent {
        width: img_type.width(),
        height: img_type.height(),
        depth: img_type.depth(),
    }
}
