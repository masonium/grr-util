//! Axis-aligned box
use super::common::*;
use crate::vertex::GrrVertex;
use nalgebra_glm as glm;

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
            0, 1, 2, 0, 2, 3, // back (-Z)
            4, 5, 6, 4, 6, 7, // front (+Z)
            8, 9, 10, 8, 10, 11, // bottom (-Y)
            12, 13, 14, 12, 14, 15, // top (+Y)
            16, 17, 18, 16, 18, 19, // left (-X)
            20, 21, 22, 20, 22, 23, // right (+X)
        ];
        let ibuff = unsafe { device.create_buffer_from_host(&index, grr::MemoryFlags::empty())? };
        let mut v = vec![
            V3::new(-half_widths.x, -half_widths.y, -half_widths.z),
            V3::new(-half_widths.x, half_widths.y, -half_widths.z),
            V3::new(half_widths.x, half_widths.y, -half_widths.z),
            V3::new(half_widths.x, -half_widths.y, -half_widths.z),
            V3::new(-half_widths.x, half_widths.y, half_widths.z),
            V3::new(-half_widths.x, -half_widths.y, half_widths.z),
            V3::new(half_widths.x, -half_widths.y, half_widths.z),
            V3::new(half_widths.x, half_widths.y, half_widths.z),
            V3::new(-half_widths.x, -half_widths.y, half_widths.z),
            V3::new(-half_widths.x, -half_widths.y, -half_widths.z),
            V3::new(half_widths.x, -half_widths.y, -half_widths.z),
            V3::new(half_widths.x, -half_widths.y, half_widths.z),
            V3::new(-half_widths.x, half_widths.y, -half_widths.z),
            V3::new(-half_widths.x, half_widths.y, half_widths.z),
            V3::new(half_widths.x, half_widths.y, half_widths.z),
            V3::new(half_widths.x, half_widths.y, -half_widths.z),
            V3::new(-half_widths.x, half_widths.y, -half_widths.z),
            V3::new(-half_widths.x, -half_widths.y, -half_widths.z),
            V3::new(-half_widths.x, -half_widths.y, half_widths.z),
            V3::new(-half_widths.x, half_widths.y, half_widths.z),
            V3::new(half_widths.x, -half_widths.y, -half_widths.z),
            V3::new(half_widths.x, half_widths.y, -half_widths.z),
            V3::new(half_widths.x, half_widths.y, half_widths.z),
            V3::new(half_widths.x, -half_widths.y, half_widths.z),
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

/// UnitCube with normals.
pub struct UnitCube<'device> {
    vbuff: Buffer,
    ibuff: Buffer,
    varr: VertexArray,
    device: &'device Device,
}

impl<'device> UnitCube<'device> {
    /// Build the vertex and index buffers that are used for any new
    /// creation.
    fn build_buffers(device: &grr::Device) -> Result<(grr::Buffer, grr::Buffer), grr::Error> {
        let index_base: [u8; 6] = [0, 1, 2, 0, 2, 3];

        let v_base = vec![
            V3::new(-1.0, -1.0, -1.0),
            V3::new(-1.0, 1.0, -1.0),
            V3::new(1.0, 1.0, -1.0),
            V3::new(1.0, -1.0, -1.0),
        ];
        let normal_base = V3::new(0.0, 0.0, -1.0);

        let rotations = [
            glm::rotation(0.0, &V3::new(1.0, 0.0, 0.0)),
            glm::rotation(std::f32::consts::PI, &V3::new(1.0, 0.0, 0.0)),
            glm::rotation(std::f32::consts::FRAC_PI_2, &V3::new(1.0, 0.0, 0.0)),
            glm::rotation(-std::f32::consts::FRAC_PI_2, &V3::new(1.0, 0.0, 0.0)),
            glm::rotation(std::f32::consts::FRAC_PI_2, &V3::new(0.0, 1.0, 0.0)),
            glm::rotation(-std::f32::consts::FRAC_PI_2, &V3::new(0.0, 1.0, 0.0)),
        ];
        let mut vn: Vec<V3> = Vec::new();
        let mut index: Vec<u8> = Vec::new();
        for (i, rot) in rotations.iter().enumerate() {
            // push the vertices
            for v in &v_base {
                vn.push(rot.transform_vector(v));
                vn.push(rot.transform_vector(&normal_base));
            }
            index.extend(index_base.iter().map(|x| x + (i * 4) as u8));
        }
        let ibuff = unsafe { device.create_buffer_from_host(&index, grr::MemoryFlags::empty())? };
        let vbuff = unsafe {
            device.create_buffer_from_host(grr::as_u8_slice(&vn), grr::MemoryFlags::empty())?
        };
        Ok((vbuff, ibuff))
    }

    pub fn new<'a: 'device>(device: &'a Device) -> Result<UnitCube<'device>, grr::Error> {
        let (vbuff, ibuff) = Self::build_buffers(device)?;

        let attribs = [
            grr::VertexAttributeDesc {
                binding: 0,
                format: grr::VertexFormat::Xyz32Float,
                location: 0,
                offset: 0,
            },
            grr::VertexAttributeDesc {
                binding: 0,
                format: grr::VertexFormat::Xyz32Float,
                location: 1,
                offset: 12,
            },
        ];

        let varr = unsafe { device.create_vertex_array(&attribs)? };
        unsafe {
            device.bind_index_buffer(varr, ibuff);
            device.bind_vertex_buffers(
                varr,
                0,
                &[grr::VertexBufferView {
                    buffer: vbuff,
                    offset: 0,
                    stride: 24,
                    input_rate: grr::InputRate::Vertex,
                }],
            );
        }

        Ok(UnitCube {
            device,
            vbuff,
            ibuff,
            varr,
        })
    }

    pub fn new_instanced<'a: 'device, V: GrrVertex>(
        device: &'a Device,
        instance_buffer: grr::Buffer,
    ) -> Result<UnitCube<'device>, grr::Error> {
        let (vbuff, ibuff) = Self::build_buffers(device)?;

        let mut attribs = vec![
            grr::VertexAttributeDesc {
                binding: 0,
                format: grr::VertexFormat::Xyz32Float,
                location: 0,
                offset: 0,
            },
            grr::VertexAttributeDesc {
                binding: 0,
                format: grr::VertexFormat::Xyz32Float,
                location: 1,
                offset: 12,
            },
        ];

        attribs.extend(<V as GrrVertex>::attribs(1, 2));

        let varr = unsafe { device.create_vertex_array(&attribs)? };
        unsafe {
            device.bind_index_buffer(varr, ibuff);
            device.bind_vertex_buffers(
                varr,
                0,
                &[
                    grr::VertexBufferView {
                        buffer: vbuff,
                        offset: 0,
                        stride: 24,
                        input_rate: grr::InputRate::Vertex,
                    },
                    <V as GrrVertex>::view(
                        instance_buffer,
                        grr::InputRate::Instance { divisor: 1 },
                    ),
                ],
            );
        }

        Ok(UnitCube {
            device,
            vbuff,
            ibuff,
            varr,
        })
    }

    pub fn draw(&self, r: std::ops::Range<u32>) {
        unsafe {
            self.device.bind_vertex_array(self.varr);
            self.device
                .draw_indexed(grr::Primitive::Triangles, grr::IndexTy::U8, 0..36, r, 0);
        }
    }
}

impl<'device> Drop for UnitCube<'device> {
    fn drop(&mut self) {
        unsafe {
            self.device.delete_vertex_array(self.varr);
            self.device.delete_buffers(&[self.vbuff, self.ibuff]);
        }
    }
}
