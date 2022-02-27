use std::collections::HashMap;

use crate::geom::Geometry;
use glam::f32::Vec3;
use serde::Deserialize;

mod settings;
use settings::Settings;

mod texture;
pub use texture::*;

use super::pass::CustomUniform;

#[derive(Deserialize, Debug)]
pub enum ObjectKind {
    #[serde(rename = "geometry")]
    Geometry(Geometry),
}

const fn default_object_scale() -> Vec3 {
    Vec3::ONE
}

#[derive(Deserialize, Debug)]
pub struct Object {
    pub name: String,
    pub kind: ObjectKind,
    #[serde(default)]
    pub position: Vec3,
    #[serde(default = "default_object_scale")]
    pub scale: Vec3,
    #[serde(default)]
    pub rotation: Vec3,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TextureRef {
    Name(String),
    Renamed { name: String, r#as: String },
}

#[derive(Debug, Deserialize)]
pub struct PassTargetBuffer {
    pub color: Vec<TextureRef>,
    #[serde(default)]
    pub depth: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PassTarget {
    Frame,
    Buffer(PassTargetBuffer),
}

impl Default for PassTarget {
    fn default() -> Self {
        PassTarget::Frame
    }
}

#[derive(Debug, Deserialize)]
pub struct Pass {
    pub vertex_shader: String,
    pub fragment_shader: String,
    #[serde(default)]
    pub objects: Vec<String>,
    #[serde(default)]
    pub textures: Vec<TextureRef>,
    #[serde(default)]
    pub target: PassTarget,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub uniforms: HashMap<String, CustomUniform>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum CameraKind {
    Orbital { distance: f32, center: Vec3 },
    //Flying { speed: f32 },
    Lookat { from: Vec3, to: Vec3, up: Vec3 },
}

impl Default for CameraKind {
    fn default() -> Self {
        CameraKind::Orbital {
            distance: 10.0,
            center: Vec3::ZERO,
        }
    }
}

fn default_mouse_fov() -> f32 {
    60.0
}

fn default_mouse_sensitifity() -> f32 {
    10.0
}

#[derive(Deserialize, Debug)]
pub struct Camera {
    #[serde(default = "default_mouse_sensitifity")]
    pub mouse_sensitivity: f32,
    #[serde(default = "default_mouse_fov")]
    pub fov: f32,
    #[serde(default)]
    pub kind: CameraKind,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            mouse_sensitivity: 10.0,
            fov: 60.0,
            kind: CameraKind::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub objects: Vec<Object>,
    #[serde(default)]
    pub passes: Vec<Pass>,
    #[serde(default)]
    pub camera: Camera,
    #[serde(default)]
    pub textures: Vec<Texture>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Color {
    RGB { r: f32, g: f32, b: f32 },
    RGBA { r: f32, g: f32, b: f32, a: f32 },
    HSV { h: f32, s: f32, v: f32 },
}
