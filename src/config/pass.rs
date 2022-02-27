use std::{collections::HashMap, fmt::Write};

use anyhow::{Context, Result};
use egui::Vec2;
use glam::{Mat4, Vec3, Vec4};
use glium::{
    program::Uniform,
    uniforms::{AsUniformValue, UniformType},
    Display, DrawParameters, Program,
};
use serde::Deserialize;

use super::{ser, Config, LoadedTarget, Shader};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BuiltinUniform {
    Model,
    View,
    Perspective,
    Time,
    MouseX,
    MouseY,
    MousePos,
    WindowWidth,
    WindowHeight,
    WindowSize,
}

impl BuiltinUniform {
    pub fn label(&self) -> &'static str {
        match *self {
            BuiltinUniform::Model => "Model",
            BuiltinUniform::View => "View",
            BuiltinUniform::Perspective => "Perspective",
            BuiltinUniform::Time => "Time",
            BuiltinUniform::MouseX => "Mouse X",
            BuiltinUniform::MouseY => "Mouse Y",
            BuiltinUniform::MousePos => "Mouse Position",
            BuiltinUniform::WindowWidth => "Window Width",
            BuiltinUniform::WindowHeight => "Window Height",
            BuiltinUniform::WindowSize => "Window Size",
        }
    }

    pub fn valid_for_uniform_type(ty: UniformType) -> &'static [BuiltinUniform] {
        match ty {
            UniformType::Float => &[
                BuiltinUniform::Time,
                BuiltinUniform::MouseX,
                BuiltinUniform::MouseY,
                BuiltinUniform::WindowWidth,
                BuiltinUniform::WindowHeight,
            ],
            UniformType::FloatVec2 => &[BuiltinUniform::MousePos, BuiltinUniform::WindowSize],
            UniformType::FloatMat4 => &[
                BuiltinUniform::Model,
                BuiltinUniform::View,
                BuiltinUniform::Perspective,
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

impl AsUniformValue for CustomUniform {
    fn as_uniform_value(&self) -> glium::uniforms::UniformValue<'_> {
        use glium::uniforms::UniformValue;

        match *self {
            CustomUniform::Mat4(x) => UniformValue::Mat4(x.to_cols_array_2d()),
            CustomUniform::Vec4(x) => UniformValue::Vec4(x.into()),
            CustomUniform::Vec3(x) => UniformValue::Vec3(x.into()),
            CustomUniform::Vec2(x) => UniformValue::Vec2(x.into()),
            CustomUniform::Float(x) => UniformValue::Float(x),
        }
    }
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
    Builtin(BuiltinUniform),
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
            ("view", UniformType::FloatMat4) => UniformBinding::Builtin(BuiltinUniform::View),
            ("projection", UniformType::FloatMat4) => {
                UniformBinding::Builtin(BuiltinUniform::Perspective)
            }
            ("model", UniformType::FloatMat4) => UniformBinding::Builtin(BuiltinUniform::Model),
            ("time", UniformType::Float) => UniformBinding::Builtin(BuiltinUniform::Time),
            ("mouse_x", UniformType::Float) => UniformBinding::Builtin(BuiltinUniform::MouseX),
            ("mouse_y", UniformType::Float) => UniformBinding::Builtin(BuiltinUniform::MouseY),
            ("window_width", UniformType::Float) => {
                UniformBinding::Builtin(BuiltinUniform::WindowWidth)
            }
            ("window_height", UniformType::Float) => {
                UniformBinding::Builtin(BuiltinUniform::WindowHeight)
            }
            ("mouse_pos", UniformType::FloatVec2) => {
                UniformBinding::Builtin(BuiltinUniform::MousePos)
            }
            ("window_size", UniformType::FloatVec2) => {
                UniformBinding::Builtin(BuiltinUniform::WindowSize)
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
