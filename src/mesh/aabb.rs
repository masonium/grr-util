//! Axis-aligned box
use super::common::*;

pub struct AABB<'device> {
    vbuff: Buffer,
    ibuff: Buffer,
    varr: VertexArray,
    device: &'device Device,
}

impl<'device> AABB<'device> {
    pub fn unit<'a: 'device>(
        device: &'a Device,
        with_tex: bool,
    ) -> Result<AABB<'device>, grr::Error> {
        Self::new(device, &V3::new(1.0, 1.0, 1.0), with_tex)
    }

    pub fn new<'a: 'device>(
        device: &'a Device,
        half_widths: &V3,
        with_tex: bool,
    ) -> Result<AABB<'device>, grr::Error> {
        let index: [u8; 36] = [
            0, 1, 2, 2, 1, 3, // back
            4, 5, 6, 6, 5, 7, // front
            8, 9, 10, 10, 9, 11, // left
            12, 13, 14, 14, 13, 15, // right
            16, 17, 18, 18, 17, 19, // bottom
            20, 21, 22, 22, 21, 23, // top
        ];
        let ibuff = unsafe { device.create_buffer_from_host(&index, grr::MemoryFlags::empty())? };
        let mut v = vec![
            V3::new(-half_widths.x, -half_widths.y, -half_widths.z),
            V3::new(-half_widths.x, half_widths.y, -half_widths.z),
            V3::new(half_widths.x, -half_widths.y, -half_widths.z),
            V3::new(half_widths.x, half_widths.y, -half_widths.z),
            V3::new(-half_widths.x, -half_widths.y, half_widths.z),
            V3::new(half_widths.x, -half_widths.y, half_widths.z),
            V3::new(-half_widths.x, half_widths.y, half_widths.z),
            V3::new(half_widths.x, half_widths.y, half_widths.z),
        ];
        if with_tex {
            v.extend_from_slice(&[
                V3::new(-1.0, -1.0, -1.0),
                V3::new(1.0, -1.0, -1.0),
                V3::new(-1.0, 1.0, -1.0),
                V3::new(1.0, 1.0, -1.0),
                V3::new(-1.0, -1.0, 1.0),
                V3::new(1.0, -1.0, 1.0),
                V3::new(-1.0, 1.0, 1.0),
                V3::new(1.0, 1.0, 1.0),
            ]);
        }

        let vbuff = unsafe {
            device.create_buffer_from_host(grr::as_u8_slice(&v), grr::MemoryFlags::empty())?
        };

        let mut attribs = vec![grr::VertexAttributeDesc {
            binding: 0,
            format: grr::VertexFormat::Xyz32Float,
            location: 0,
            offset: 0,
        }];
        if with_tex {
            attribs.push(grr::VertexAttributeDesc {
                binding: 0,
                format: grr::VertexFormat::Xyz32Float,
                location: 1,
                offset: (8 * std::mem::size_of_val(&v[0])) as _,
            });
        }

        let varr = unsafe { device.create_vertex_array(&attribs)? };
        unsafe {
            device.bind_index_buffer(varr, ibuff);
            device.bind_vertex_buffers(
                varr,
                0,
                &[grr::VertexBufferView {
                    buffer: vbuff,
                    offset: 0,
                    stride: std::mem::size_of_val(&v[0]) as _,
                    input_rate: grr::InputRate::Vertex,
                }],
            );
        }

        Ok(AABB {
            device,
            vbuff,
            ibuff,
            varr,
        })
    }

    pub fn draw(&self) {
        unsafe {
            self.device.bind_vertex_array(self.varr);
            self.device
                .draw_indexed(grr::Primitive::Triangles, grr::IndexTy::U8, 0..36, 0..1, 0);
        }
    }
}

impl<'device> Drop for AABB<'device> {
    fn drop(&mut self) {
        unsafe {
            self.device.delete_vertex_array(self.varr);
            self.device.delete_buffers(&[self.vbuff, self.ibuff]);
        }
    }
}
