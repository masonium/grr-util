//! Full-screen quad mesh
use crate::mesh::common::*;

/// Full-screen quad, with XY coordinates ranging [-1, 1] ^ 2
pub struct Quad<'device> {
    vbuff: Buffer,
    ibuff: Buffer,
    varr: VertexArray,
    device: &'device Device,
}

impl<'device> Quad<'device> {
    pub fn new(device: &grr::Device) -> Result<Quad, grr::Error> {
        let pos_data = vec![-1.0_f32, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0];
        let vbuff = unsafe {
            device
                .create_buffer_from_host(grr::as_u8_slice(&pos_data), grr::MemoryFlags::empty())?
        };
        let ibuff = unsafe {
            device.create_buffer_from_host(
                grr::as_u8_slice(&[0u32, 1, 2, 0, 2, 3]),
                grr::MemoryFlags::empty(),
            )?
        };

        let varr = unsafe {
            device.create_vertex_array(&[grr::VertexAttributeDesc {
                location: 0,
                binding: 0,
                format: grr::VertexFormat::Xy32Float,
                offset: 0,
            }])?
        };

        unsafe {
            device.bind_vertex_buffers(
                varr,
                0,
                &[grr::VertexBufferView {
                    buffer: vbuff,
                    stride: 8,
                    offset: 0,
                    input_rate: grr::InputRate::Vertex,
                }],
            );
            device.bind_index_buffer(varr, ibuff);
        }

        Ok(Quad {
            vbuff,
            ibuff,
            varr,
            device,
        })
    }

    pub fn draw(&self) {
        unsafe {
            self.device.bind_vertex_array(self.varr);
            self.device
                .draw_indexed(grr::Primitive::Triangles, grr::IndexTy::U32, 0..6, 0..1, 0);
        }
    }
}

impl<'device> Drop for Quad<'device> {
    fn drop(&mut self) {
        unsafe {
            self.device.delete_vertex_arrays(&[self.varr]);
            self.device.delete_buffers(&[self.vbuff, self.ibuff]);
        }
    }
}
