//! Parameterized, n-segment line.
use super::common::*;
use std::iter::FromIterator;

/// Create a draw-able mesh containing a regular line segment.
/// Vertices are X-position only, with N+1 evenly-spaced vertices from 0 to 1.
pub struct Line<'device> {
    vbuff: Buffer,
    varr: VertexArray,
    n: usize, // number of line_segments
    device: &'device Device,
}

impl<'device> Line<'device> {
    /// Create a line-mesh consisting of `num_line_pieces` segments.
    pub fn new(device: &grr::Device, num_line_pieces: usize) -> Result<Line, grr::Error> {
        let vertices: Vec<f32> = Vec::from_iter(linspace(0.0, 1.0, (num_line_pieces + 1) as usize));

        let vbuff = unsafe {
            device
                .create_buffer_from_host(grr::as_u8_slice(&vertices), grr::MemoryFlags::empty())?
        };

        let varr = unsafe {
            device.create_vertex_array(&[grr::VertexAttributeDesc {
                location: 0,
                binding: 0,
                format: grr::VertexFormat::X32Float,
                offset: 0,
            }])?
        };

        unsafe {
            device.bind_vertex_buffers(
                varr,
                0,
                &[grr::VertexBufferView {
                    buffer: vbuff,
                    stride: 4,
                    offset: 0,
                    input_rate: grr::InputRate::Vertex,
                }],
            );
        }

        Ok(Line {
            vbuff,
            varr,
            device,
            n: num_line_pieces,
        })
    }

    pub fn draw(&self) {
        unsafe {
            self.device.bind_vertex_array(self.varr);
            self.device
                .draw(grr::Primitive::LineStrip, 0..(self.n as u32 + 1), 0..1);
        }
    }
}

impl<'device> Drop for Line<'device> {
    fn drop(&mut self) {
        unsafe {
            self.device.delete_vertex_arrays(&[self.varr]);
            self.device.delete_buffers(&[self.vbuff]);
        }
    }
}
