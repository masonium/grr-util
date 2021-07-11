//! Full-screen quad mesh
use std::marker::PhantomData;

use grr::MemoryFlags;

use crate::{mesh::common::*, GrrVertex};

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
                grr::as_u8_slice(&[0u32, 2, 1, 0, 3, 2]),
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

/// Full-screen quad, with XY coordinates ranging [-1, 1] ^ 2
pub struct InstancedQuad<'device, T: GrrVertex> {
    vbuff: Buffer,
    ibuff: Buffer,
    instance_buffer: Buffer,
    num_instances: u32,
    capacity_instances: u32,
    //instances: Vec<T>,
    varr: VertexArray,
    device: &'device Device,
    _data: PhantomData<T>,
}

impl<'device, T: GrrVertex> InstancedQuad<'device, T> {
    pub fn new(device: &'device grr::Device, num_instances_init: u32) -> Result<Self, grr::Error> {
        let pos_data = vec![-1.0_f32, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0];
        let vbuff = unsafe {
            device
                .create_buffer_from_host(grr::as_u8_slice(&pos_data), grr::MemoryFlags::empty())?
        };
        let ibuff = unsafe {
            device.create_buffer_from_host(
                grr::as_u8_slice(&[0u32, 2, 1, 0, 3, 2]),
                grr::MemoryFlags::empty(),
            )?
        };
        let instbuff = unsafe {
            device.create_buffer(
                std::mem::size_of::<T>() as u64 * num_instances_init as u64,
                grr::MemoryFlags::CPU_MAP_WRITE | grr::MemoryFlags::DYNAMIC,
            )?
        };

        let mut attribs = vec![grr::VertexAttributeDesc {
            location: 0,
            binding: 0,
            format: grr::VertexFormat::Xy32Float,
            offset: 0,
        }];
        attribs.extend_from_slice(&T::attribs(1, 1));

        let varr = unsafe { device.create_vertex_array(&attribs)? };

        unsafe {
            device.bind_vertex_buffers(
                varr,
                0,
                &[
                    grr::VertexBufferView {
                        buffer: vbuff,
                        stride: 8,
                        offset: 0,
                        input_rate: grr::InputRate::Vertex,
                    },
                    grr::VertexBufferView {
                        buffer: instbuff,
                        stride: std::mem::size_of::<T>() as u32,
                        offset: 0,
                        input_rate: grr::InputRate::Instance { divisor: 1 },
                    },
                ],
            );
            device.bind_index_buffer(varr, ibuff);
        }

        Ok(InstancedQuad {
            vbuff,
            ibuff,
            instance_buffer: instbuff,
            num_instances: num_instances_init,
            capacity_instances: num_instances_init,
            varr,
            device,
            _data: PhantomData {},
        })
    }

    /// Perform the equivalent draw command as the last one.
    pub fn draw(&self) {
        unsafe {
            self.device.bind_vertex_array(self.varr);
            self.device.draw_indexed(
                grr::Primitive::Triangles,
                grr::IndexTy::U32,
                0..6,
                0..self.num_instances,
                0,
            );
        }
    }

    /// Update the instances to render.
    ///
    /// Internally update the storage buffer if the new collection of
    /// instances is larger than what has previously been allocated.
    pub fn update_instances(&mut self, instances: &[T]) {
        // If we don't have room in the existing instance buffer for
        // the new quads, generate a new buffer and bind it to our VAO.
        if (instances.len() as u32) > self.capacity_instances {
            unsafe {
                let new_buff = self
                    .device
                    .create_buffer_from_host(
                        grr::as_u8_slice(instances),
                        MemoryFlags::DYNAMIC | MemoryFlags::CPU_MAP_WRITE,
                    )
                    .unwrap();
                self.device.delete_buffer(self.instance_buffer);
                self.instance_buffer = new_buff;
                self.device.bind_vertex_buffers(
                    self.varr,
                    1,
                    &[grr::VertexBufferView {
                        buffer: new_buff,
                        stride: std::mem::size_of::<T>() as u32,
                        offset: 0,
                        input_rate: grr::InputRate::Instance { divisor: 1 },
                    }],
                );
            }
            self.capacity_instances = instances.len() as u32;
        } else {
            unsafe {
                self.device.copy_host_to_buffer(
                    self.instance_buffer,
                    0,
                    grr::as_u8_slice(instances),
                );
                self.num_instances = instances.len() as u32;
            }
        }
    }
}

impl<'device, T: GrrVertex> Drop for InstancedQuad<'device, T> {
    fn drop(&mut self) {
        unsafe {
            self.device.delete_vertex_arrays(&[self.varr]);
            self.device
                .delete_buffers(&[self.vbuff, self.ibuff, self.instance_buffer]);
        }
    }
}
