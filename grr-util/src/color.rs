//! Color utils

/// Given a hexadecimal color, return a grr::Constant that represents
/// the RGB color.  For the base function, the input color is assumed to
/// be in Srgb, and the result is assumed to be linear.
pub fn hex_constant_rgb(s: &str) -> Option<grr::Constant> {
    let srgb = art_util::parse_hex_srgb(s)?;

    let c = srgb.into_linear().into_components();
    Some(grr::Constant::Vec3([c.0, c.1, c.2]))
}

/// Given a hexadecimal color, return a grr::Constant that represents
/// the RGBA color.  For the base function, the input color is assumed
/// to be in Srgb, and the result is assumed to be linear. The alpha
/// is assumed to be 1.0.
pub fn hex_constant_rgba(s: &str) -> Option<grr::Constant> {
    let srgb = art_util::parse_hex_srgb(s)?;

    let c = srgb.into_linear().into_components();
    Some(grr::Constant::Vec4([c.0, c.1, c.2, 1.0]))
}
