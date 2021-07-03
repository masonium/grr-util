//! Capture screenshots from a framebuffer or texture to disk.
use std::path::Path;

/// Save the color information for the default framebuffer to disk.
pub fn save_framebuffer_rgba<P: AsRef<Path>>(
    src_framebuffer: grr::Framebuffer,
    device: &grr::Device,
    region: grr::Region,
    path: P,
) -> Result<(), std::io::Error> {
    let pixel_data_size: usize = (region.w * region.h) as usize * 4;

    // create a non-multisampled framebuffer to blit to.
    let (img, img_view) = unsafe {
        device
            .create_image_and_view(
                grr::ImageType::D2 {
                    width: region.w as u32,
                    height: region.h as u32,
                    layers: 1,
                    samples: 1,
                },
                grr::Format::R8G8B8A8_SRGB,
                1,
            )
            .unwrap()
    };

    let fb = unsafe { device.create_framebuffer().unwrap() };
    unsafe {
        device.bind_attachments(
            fb,
            &[(
                grr::Attachment::Color(0),
                grr::AttachmentView::Image(img_view),
            )],
        );
    }

    // create a buffer to read the results to.
    let mut buffer_data: Vec<u8> = Vec::with_capacity(pixel_data_size);
    buffer_data.resize(pixel_data_size, 0);

    unsafe {
        // blit to the target, non-multisample buffer
        device.blit(
            src_framebuffer,
            region,
            fb,
            grr::Region {
                x: 0,
                y: 0,
                w: region.w,
                h: region.h,
            },
            grr::Filter::Linear,
        );

        device.bind_read_framebuffer(fb);

        let memory_layout = grr::MemoryLayout {
            alignment: 1,
            base_format: grr::BaseFormat::RGBA,
            format_layout: grr::FormatLayout::U8,
            row_length: 0,
            image_height: 0,
        };

        // copy from that framebuffer to host memory
        device.copy_attachment_to_host(region, memory_layout, &mut buffer_data);
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

    unsafe {
        device.delete_framebuffer(fb);
        device.delete_image_view(img_view);
        device.delete_image(img);
    }
    Ok(())
}
