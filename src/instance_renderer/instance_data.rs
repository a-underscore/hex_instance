use hex::vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex, Copy, Clone)]
#[repr(C)]
pub struct InstanceData {
    #[format(R32G32B32A32_SFLOAT)]
    pub color: [f32; 4],
    #[format(R32G32B32_SFLOAT)]
    pub transform_x: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub transform_y: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub transform_z: [f32; 3],
}
