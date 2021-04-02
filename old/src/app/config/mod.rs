use crate::app::{
    camera::{Camera as Cam, CameraHandler, OrbitalCamera},
    resource::AnyResourceId,
    Resource, Resources, Vertex,
};
use anyhow::{Context, Result};
use glam::f32::{Vec2, Vec3};
use glium::{Display, IndexBuffer, VertexBuffer};
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Read, sync::Arc};

mod geom;
pub use geom::Geometry;

mod pass;
pub use pass::{LoadedPass, Pass};

#[derive(Debug, Serialize, Deserialize)]
pub enum ObjectType {
    #[serde(rename = "geometry")]
    Geometry(Geometry),
}

fn scale() -> Vec3 {
    Vec3::ONE
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    name: String,
    #[serde(rename = "type")]
    ty: ObjectType,
    #[serde(default)]
    position: Vec3,
    #[serde(default = "scale")]
    scale: Vec3,
    #[serde(default)]
    rotation: Vec3,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CameraType {
    Orbital,
    Fps,
}

impl Default for CameraType {
    fn default() -> Self {
        CameraType::Orbital
    }
}

fn fov() -> f32 {
    90.0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Camera {
    #[serde(rename = "type")]
    #[serde(default)]
    ty: CameraType,
    #[serde(default = "fov")]
    fov: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            ty: CameraType::default(),
            fov: 90.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    camera: Camera,
    #[serde(default)]
    objects: Vec<Object>,
    #[serde(default)]
    passes: Vec<Pass>,
}

pub struct Shader {
    source: String,
}

impl Resource for Shader {
    fn load(mut file: File, _: &Display, _: &mut Resources) -> Result<Shader> {
        let mut source = String::new();
        file.read_to_string(&mut source)
            .context("loading shader source")?;
        Ok(Shader { source })
    }
}

pub struct LoadedObject {
    verticies: VertexBuffer<Vertex>,
    indicies: IndexBuffer<u32>,
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
}

impl LoadedObject {
    pub fn load(object: &Object, display: &Display) -> Result<Self> {
        let (verticies, indicies) = match object.ty {
            ObjectType::Geometry(ref x) => x.to_buffers(display)?,
        };
        Ok(LoadedObject {
            verticies,
            indicies,
            position: object.position,
            rotation: object.rotation,
            scale: object.scale,
        })
    }
}

pub struct LoadedConfig {
    pub objects: HashMap<String, Arc<LoadedObject>>,
    pub passes: Vec<LoadedPass>,
    pub camera_handler: Box<dyn CameraHandler>,
    pub camera: Cam,
}

impl Resource for LoadedConfig {
    fn load(file: File, display: &Display, res: &mut Resources) -> Result<LoadedConfig> {
        let config: Config =
            serde_json::from_reader(file).context("Failed to parse config file")?;

        let mut objects = HashMap::new();
        for object in config.objects {
            let load = LoadedObject::load(&object, display)?;
            objects.insert(object.name, Arc::new(load));
        }

        let mut passes = Vec::new();
        for (idx, p) in config.passes.iter().enumerate() {
            passes.push(
                LoadedPass::new(p, &objects, display, res)
                    .with_context(|| format!("Failed to load pass {} ", idx))?,
            );
        }
        let mut camera_handler = match config.camera.ty {
            CameraType::Orbital => Box::new(OrbitalCamera::new()) as Box<dyn CameraHandler>,
            CameraType::Fps => todo!(),
        };

        let mut camera = Cam::new();
        camera_handler.handle_mouse_move(&mut camera, Vec2::ZERO);

        Ok(LoadedConfig {
            passes,
            objects,
            camera: Cam::new(),
            camera_handler,
        })
    }

    fn reload(&mut self, file: File, display: &Display, res: &mut Resources) -> Result<()> {
        let mut new_value = Self::load(file, display, res)?;

        new_value
            .camera_handler
            .handle_mouse_move(&mut new_value.camera, Vec2::ZERO);

        *self = new_value;
        Ok(())
    }

    fn reload_dependency(
        &mut self,
        dependency: AnyResourceId,
        display: &Display,
        res: &Resources,
    ) -> Result<bool> {
        for (idx, p) in self.passes.iter_mut().enumerate() {
            p.reload(dependency, display, res)
                .with_context(|| format!("Failed to reload pass {} ", idx))?;
        }
        Ok(false)
    }
}
