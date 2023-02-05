use hex::glium::implement_vertex;

#[derive(Copy, Clone)]
pub struct InstanceData {
    pub z: f32,
    pub transform: [[f32; 3]; 3],
    pub color: [f32; 4],
    pub id: f32,
}

implement_vertex!(InstanceData, z, transform, color, id);
