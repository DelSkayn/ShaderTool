use glium::implement_vertex;

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

implement_vertex!(Vertex, position, normal, tex_coord);
