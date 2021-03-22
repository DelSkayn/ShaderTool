use crate::app::Vertex;
use anyhow::Result;
use glium::{Display, IndexBuffer, VertexBuffer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Cube {
    #[serde(default = "one")]
    width: f32,
    #[serde(default = "one")]
    height: f32,
    #[serde(default = "one")]
    depth: f32,
}

impl Cube {
    pub fn to_buffers(
        &self,
        display: &Display,
    ) -> Result<(VertexBuffer<Vertex>, IndexBuffer<u32>)> {
        let x = self.width / 2.0;
        let y = self.height / 2.0;
        let z = self.depth / 2.0;

        let positions = [
            [-x, -y, -z],
            [x, -y, -z],
            [-x, y, -z],
            [x, y, -z],
            [-x, -y, z],
            [x, -y, z],
            [-x, y, z],
            [x, y, z],
        ];

        let normals = [
            [0.0, 0.0, -1.0],
            [0.0, 0.0, 1.0],
            [0.0, -1.0, 0.0],
            [0.0, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ];

        let tex_coords = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];

        let verticies = &[
            Vertex {
                position: positions[0],
                normal: normals[0],
                tex_coord: tex_coords[0],
            },
            Vertex {
                position: positions[1],
                normal: normals[0],
                tex_coord: tex_coords[1],
            },
            Vertex {
                position: positions[3],
                normal: normals[0],
                tex_coord: tex_coords[3],
            },
            Vertex {
                position: positions[2],
                normal: normals[0],
                tex_coord: tex_coords[2],
            },
            Vertex {
                position: positions[5],
                normal: normals[1],
                tex_coord: tex_coords[0],
            },
            Vertex {
                position: positions[4],
                normal: normals[1],
                tex_coord: tex_coords[1],
            },
            Vertex {
                position: positions[6],
                normal: normals[1],
                tex_coord: tex_coords[3],
            },
            Vertex {
                position: positions[7],
                normal: normals[1],
                tex_coord: tex_coords[2],
            },
            Vertex {
                position: positions[4],
                normal: normals[2],
                tex_coord: tex_coords[0],
            },
            Vertex {
                position: positions[5],
                normal: normals[2],
                tex_coord: tex_coords[1],
            },
            Vertex {
                position: positions[1],
                normal: normals[2],
                tex_coord: tex_coords[3],
            },
            Vertex {
                position: positions[0],
                normal: normals[2],
                tex_coord: tex_coords[2],
            },
            Vertex {
                position: positions[2],
                normal: normals[3],
                tex_coord: tex_coords[0],
            },
            Vertex {
                position: positions[3],
                normal: normals[3],
                tex_coord: tex_coords[1],
            },
            Vertex {
                position: positions[7],
                normal: normals[3],
                tex_coord: tex_coords[3],
            },
            Vertex {
                position: positions[6],
                normal: normals[3],
                tex_coord: tex_coords[2],
            },
            Vertex {
                position: positions[4],
                normal: normals[4],
                tex_coord: tex_coords[0],
            },
            Vertex {
                position: positions[0],
                normal: normals[4],
                tex_coord: tex_coords[1],
            },
            Vertex {
                position: positions[2],
                normal: normals[4],
                tex_coord: tex_coords[3],
            },
            Vertex {
                position: positions[6],
                normal: normals[4],
                tex_coord: tex_coords[2],
            },
            Vertex {
                position: positions[1],
                normal: normals[5],
                tex_coord: tex_coords[0],
            },
            Vertex {
                position: positions[5],
                normal: normals[5],
                tex_coord: tex_coords[1],
            },
            Vertex {
                position: positions[7],
                normal: normals[5],
                tex_coord: tex_coords[3],
            },
            Vertex {
                position: positions[3],
                normal: normals[5],
                tex_coord: tex_coords[2],
            },
        ];

        let indicies = &[0, 2, 1, 0, 3, 2];

        let mut index = Vec::new();
        for i in 0..6 {
            indicies.iter().for_each(|x| {
                index.push(x + i * 4);
            })
        }

        let vertex_buffer = VertexBuffer::immutable(display, verticies)?;
        let index_buffer = IndexBuffer::<u32>::immutable(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &index,
        )?;

        Ok((vertex_buffer, index_buffer))
    }
}

fn one() -> f32 {
    1.0
}

impl Default for Cube {
    fn default() -> Self {
        Cube {
            width: 1.0,
            height: 1.0,
            depth: 1.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Geometry {
    #[serde(rename = "screen_quad")]
    ScreenQuad,
    #[serde(rename = "cube")]
    Cube(Cube),
}

impl Geometry {
    pub fn to_buffers(
        &self,
        display: &Display,
    ) -> Result<(VertexBuffer<Vertex>, IndexBuffer<u32>)> {
        match &self {
            Geometry::Cube(ref x) => x.to_buffers(display),
            _ => todo!(),
        }
    }
}
