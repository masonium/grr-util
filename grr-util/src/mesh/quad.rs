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
    num_prims: u32,
    //instances: Vec<T>,
    varr: VertexArray,
    device: &'device Device,
    _data: PhantomData<T>,
}

/// Produce vertex (position) and index data for a 2D quadrilateral with coordinates [-1, 1]^2.
fn quad_data(num_segments: u32) -> (Vec<f32>, Vec<u32>) {
    let r = 2.0 / num_segments as f32;
    let coords: Vec<f32> = (0..num_segments+1).map(|i| -1.0 + i as f32 * r).collect();

    let mut pos_data = Vec::with_capacity(((num_segments+1) * (num_segments+1) * 2) as usize);
    // add vertices from bottom to top, left-to-right.
    for y in &coords {
	for x in &coords {
	    pos_data.push(*x);
	    pos_data.push(*y);
	}
    }
    // now construct the index data
    let mut indices = Vec::with_capacity((num_segments * num_segments * 6) as usize);

    for j in 0..num_segments {
	let base = j * (num_segments + 1);
	let row = num_segments + 1;
	for i in 0..num_segments {
	    indices.push(base + i);
	    indices.push(base + i + 1);
	    indices.push(base + i + row);
	    indices.push(base + i + row);
	    indices.push(base + i + 1);
	    indices.push(base + i + row + 1);
	}
    }

    (pos_data, indices)
}

impl<'device, T: GrrVertex> InstancedQuad<'device, T> {
    /// Create a quad consisting of n seqgments per side as triangles.
    pub fn new(device: &'device grr::Device, 
	       num_segments: u32,
	       num_instances_init: u32) -> Result<Self, grr::Error> {
	let (pos_data, indices) = quad_data(num_segments);
        //let pos_data = vec![-1.0_f32, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0];

        let vbuff = unsafe {
            device
                .create_buffer_from_host(grr::as_u8_slice(&pos_data), grr::MemoryFlags::empty())?
        };
        let ibuff = unsafe {
            device.create_buffer_from_host(
                grr::as_u8_slice(&indices),
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
	    num_prims: indices.len() as u32,
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
                0..self.num_prims,
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
