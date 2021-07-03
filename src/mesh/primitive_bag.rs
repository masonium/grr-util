//! Bag of (un-indexed) primitives
use super::common::*;
use crate::GrrVertex;

/// Collection of (un-indexed) primitives.
pub struct PrimitiveBag<'device> {
    vbuff: Buffer,
    varr: VertexArray,
    n: u32, // primitive,
    prim_type: grr::Primitive,
    device: &'device Device,
}

impl<'device> PrimitiveBag<'device> {
    pub fn new<V: GrrVertex>(
        vertices: Vec<V>,
        prim_type: grr::Primitive,
        device: &Device,
    ) -> grr::Result<PrimitiveBag> {
        let vbuff = unsafe {
            device
                .create_buffer_from_host(grr::as_u8_slice(&vertices), grr::MemoryFlags::empty())?
        };

        let varr = unsafe { device.create_vertex_array(&V::attribs(0, 0))? };

        unsafe {
            device.bind_vertex_buffers(
                varr,
                0,
                &[grr::VertexBufferView {
                    buffer: vbuff,
                    stride: std::mem::size_of::<V>() as u32,
                    offset: 0,
                    input_rate: grr::InputRate::Vertex,
                }],
            );
        }

        Ok(PrimitiveBag {
            vbuff,
            varr,
            device,
            prim_type,
            n: vertices.len() as u32,
        })
    }

    /// Draw the underlying primitive.
    pub fn draw(&self, r: std::ops::Range<u32>) {
        unsafe {
            self.device.bind_vertex_array(self.varr);
            self.device.draw(self.prim_type, 0..self.n, r);
        }
    }
}

impl<'device> Drop for PrimitiveBag<'device> {
    fn drop(&mut self) {
        unsafe {
            self.device.delete_vertex_arrays(&[self.varr]);
            self.device.delete_buffers(&[self.vbuff]);
        }
    }
}
