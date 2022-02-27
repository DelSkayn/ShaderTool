use std::{collections::HashMap, fmt::Write};

use anyhow::{Context, Result};
use egui::Vec2;
use glam::{Mat4, Vec3, Vec4};
use glium::{program::Uniform, uniforms::UniformType, Display, DrawParameters, Program};
use serde::Deserialize;

use super::{ser, Config, LoadedTarget, Shader};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BuiltInUniform {
    Model,
    View,
    Projection,
    Time,
    MouseX,
    MouseY,
    MousePos,
    WindowWidth,
    WindowHeight,
    WindowSize,
}

impl BuiltInUniform {
    pub fn label(&self) -> &'static str {
        match *self {
            BuiltInUniform::Model => "Model",
            BuiltInUniform::View => "View",
            BuiltInUniform::Projection => "Projection",
            BuiltInUniform::Time => "Time",
            BuiltInUniform::MouseX => "Mouse X",
            BuiltInUniform::MouseY => "Mouse Y",
            BuiltInUniform::MousePos => "Mouse Position",
            BuiltInUniform::WindowWidth => "Window Width",
            BuiltInUniform::WindowHeight => "Window Height",
            BuiltInUniform::WindowSize => "Window Size",
        }
    }

    pub fn valid_for_uniform_type(ty: UniformType) -> &'static [BuiltInUniform] {
        match ty {
            UniformType::Float => &[
                BuiltInUniform::Time,
                BuiltInUniform::MouseX,
                BuiltInUniform::MouseY,
                BuiltInUniform::WindowWidth,
                BuiltInUniform::WindowHeight,
            ],
            UniformType::FloatVec2 => &[BuiltInUniform::MousePos, BuiltInUniform::WindowSize],
            UniformType::FloatMat4 => &[
                BuiltInUniform::Model,
                BuiltInUniform::View,
                BuiltInUniform::Projection,
            ],
            _ => &[],
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(untagged)]
pub enum CustomUniform {
    Mat4(Mat4),
    Vec4(Vec4),
    Vec3(Vec3),
    Vec2(Vec2),
    Float(f32),
}

impl CustomUniform {
    pub fn from_uniform_type(kind: UniformType) -> Option<Self> {
        match kind {
            UniformType::FloatMat4 => Some(CustomUniform::Mat4(Default::default())),
            UniformType::FloatVec4 => Some(CustomUniform::Vec4(Default::default())),
            UniformType::FloatVec3 => Some(CustomUniform::Vec3(Default::default())),
            UniformType::FloatVec2 => Some(CustomUniform::Vec2(Default::default())),
            UniformType::Float => Some(CustomUniform::Float(Default::default())),
            _ => None,
        }
    }

    pub fn ensure_compatible(&self, kind: &UniformType) -> Result<()> {
        match self {
            CustomUniform::Mat4(_) => {
                if UniformType::FloatMat4 != *kind {
                    bail!(
                        "Invalid uniform type in config, found `FloatMat4` expected `{:?}`",
                        kind
                    );
                }
            }
            CustomUniform::Vec4(_) => {
                if UniformType::FloatVec4 != *kind {
                    bail!(
                        "Invalid uniform type in config, found `FloatVec4` expected `{:?}`",
                        kind
                    );
                }
            }
            CustomUniform::Vec3(_) => {
                if UniformType::FloatVec3 != *kind {
                    bail!(
                        "Invalid uniform type in config, found `FloatVec3` expected `{:?}`",
                        kind
                    );
                }
            }
            CustomUniform::Vec2(_) => {
                if UniformType::FloatVec2 != *kind {
                    bail!(
                        "Invalid uniform type in config, found `FloatVec2` expected `{:?}`",
                        kind
                    );
                }
            }
            CustomUniform::Float(_) => {
                if UniformType::Float != *kind {
                    bail!(
                        "Invalid uniform type in config, found `Float` expected `{:?}`",
                        kind
                    );
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UniformBinding {
    BuiltIn(BuiltInUniform),
    Custom(CustomUniform),
    Unbound,
}

#[derive(Debug)]
pub struct UniformData {
    pub kind: Uniform,
    pub binding: UniformBinding,
}

impl UniformData {
    pub fn from_name_uniform(name: &str, kind: &Uniform) -> Self {
        let binding = match (name, kind.ty) {
            ("view", UniformType::FloatMat4) => UniformBinding::BuiltIn(BuiltInUniform::View),
            ("projection", UniformType::FloatMat4) => {
                UniformBinding::BuiltIn(BuiltInUniform::Projection)
            }
            ("model", UniformType::FloatMat4) => UniformBinding::BuiltIn(BuiltInUniform::Model),
            ("time", UniformType::Float) => UniformBinding::BuiltIn(BuiltInUniform::Time),
            ("mouse_x", UniformType::Float) => UniformBinding::BuiltIn(BuiltInUniform::MouseX),
            ("mouse_y", UniformType::Float) => UniformBinding::BuiltIn(BuiltInUniform::MouseY),
            ("window_width", UniformType::Float) => {
                UniformBinding::BuiltIn(BuiltInUniform::WindowWidth)
            }
            ("window_height", UniformType::Float) => {
                UniformBinding::BuiltIn(BuiltInUniform::WindowHeight)
            }
            ("mouse_pos", UniformType::FloatVec2) => {
                UniformBinding::BuiltIn(BuiltInUniform::MousePos)
            }
            ("window_size", UniformType::Float) => {
                UniformBinding::BuiltIn(BuiltInUniform::WindowSize)
            }
            _ => UniformBinding::Unbound,
        };
        UniformData {
            kind: kind.clone(),
            binding,
        }
    }
}

#[derive(Debug)]
pub struct LoadedPass {
    pub vertex: Shader,
    pub fragment: Shader,
    pub program: Program,
    pub draw_parameters: DrawParameters<'static>,
    pub objects: Vec<usize>,
    pub textures: Vec<(usize, String)>,
    pub target: Option<LoadedTarget>,
    pub uniforms: HashMap<String, UniformData>,
}

impl Config {
    pub fn load_pass2(
        pass: &ser::Pass,
        object_name_match: &HashMap<String, usize>,
        texture_name_match: &HashMap<String, usize>,
        display: &Display,
    ) -> Result<LoadedPass> {
        let objects = pass.objects.iter().try_fold(Vec::new(), |mut acc, x| {
            if let Some(x) = object_name_match.get(x).copied() {
                acc.push(x);
            } else {
                let mut expects = String::new();
                write!(expects, "Expected one of ").unwrap();
                for (idx, k) in object_name_match.keys().enumerate() {
                    if idx != 0 {
                        write!(expects, ",").unwrap();
                    }
                    write!(expects, "`{}`", k).unwrap();
                }
                write!(expects, ".").unwrap();

                bail!("Could not find object `{}`. {}", x, expects)
            }
            Ok(acc)
        })?;

        let textures =
            pass.textures
                .iter()
                .try_fold::<_, _, Result<_>>(Vec::new(), |mut acc, x| {
                    acc.push(
                        Self::link_texture(x, texture_name_match)
                            .context("Failed to link pass texture")?,
                    );
                    Result::Ok(acc)
                })?;

        let vertex = Shader::load(&pass.vertex_shader).context("Failed to load vertex shader")?;
        let fragment =
            Shader::load(&pass.fragment_shader).context("Failed to load fragment shader")?;

        let program = Program::from_source(display, &vertex.source, &fragment.source, None)
            .context("Failed to compile program")?;

        for (name, _) in program.attributes() {
            match name.as_str() {
                "position" | "normal" | "tex_coord" => {}
                x => bail!("Invalid attribute `{}` used in shader", x,),
            }
        }

        let mut uniforms: HashMap<_, _> = program
            .uniforms()
            .map(|(a, b)| {
                let data = UniformData::from_name_uniform(a, b);
                (a.clone(), data)
            })
            .collect();

        for (name, value) in pass.uniforms.iter() {
            if let Some(x) = uniforms.get_mut(name) {
                ensure!(x.kind.size.is_none(), "Uniform arrays are not supported");
                value
                    .ensure_compatible(&x.kind.ty)
                    .with_context(|| format!("Invalid uniform binding `{}`", name))?;
                x.binding = UniformBinding::Custom(*value);
            }
        }

        let target = match pass.target {
            ser::PassTarget::Frame => None,
            ser::PassTarget::Buffer(ref x) => {
                let color = x
                    .color
                    .iter()
                    .try_fold::<_, _, Result<_>>(Vec::new(), |mut acc, x| {
                        acc.push(Self::link_texture(&x, texture_name_match)?);
                        Ok(acc)
                    })
                    .context("Failed to link pass target color attachment")?;
                let depth = x
                    .depth
                    .as_ref()
                    .map(|x| {
                        Self::link_texture(&ser::TextureRef::Name(x.clone()), texture_name_match)
                    })
                    .transpose()
                    .context("Failed to link pass target depth attachment")?
                    .map(|x| x.0);

                Some(LoadedTarget { color, depth })
            }
        };

        let draw_parameters = pass.settings.to_params();

        Ok(LoadedPass {
            vertex,
            fragment,
            objects,
            draw_parameters,
            textures,
            program,
            target,
            uniforms,
        })
    }
}
