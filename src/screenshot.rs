//! Capture screenshots from a framebuffer or texture to disk.
use std::path::Path;

/// Save the color information for the default framebuffer to disk.
pub fn save_framebuffer_rgba<P: AsRef<Path>>(
    device: &grr::Device,
    region: grr::Region,
    path: P,
) -> Result<(), std::io::Error> {
    let pixel_data_size: usize = (region.w * region.h) as usize * 4;

    // create a buffer to read the results to.
    let mut buffer_data: Vec<u8> = Vec::with_capacity(pixel_data_size);
    buffer_data.resize(pixel_data_size, 0);

    unsafe {
        device.read_pixels(
            region,
            grr::SubresourceLayout {
                alignment: 1,
                base_format: grr::BaseFormat::RGBA,
                format_layout: grr::FormatLayout::U8,
                row_pitch: 0,
                image_height: 0,
            },
            &mut buffer_data,
        );
    }

    // Swap the order of the rows so that the output will not be flipped vertically.
    for i in 0..region.h / 2 {
        let r1_offset = (i * region.w * 4) as usize;
        let r2_offset = ((region.h - 1 - i) * region.w * 4) as usize;
        unsafe {
            std::ptr::swap_nonoverlapping(
                buffer_data.as_mut_ptr().add(r1_offset),
                buffer_data.as_mut_ptr().add(r2_offset),
                (region.w * 4) as usize,
            );
        }
    }

    image::save_buffer(
        path,
        &buffer_data,
        region.w as u32,
        region.h as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();
    Ok(())
}
