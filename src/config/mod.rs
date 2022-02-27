use crate::render::Vertex;
use anyhow::{Context, Result};
use glam::f32::{Mat4, Quat, Vec2, Vec3};
use glium::glutin::event::DeviceEvent;
use glium::DrawParameters;
use glium::{
    glutin::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    Display, IndexBuffer, Program, VertexBuffer,
};
use std::{collections::HashMap, ffi::OsStr, fmt::Write, fs::File, io::Read, path::Path};

use self::ser::CameraKind;

mod ser;
mod texture;
use texture::LoadedTexture;
mod render;

#[derive(Debug)]
pub struct Shader {
    source: String,
}

impl Shader {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut source = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut source)?;
        Ok(Shader { source })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LoadedCamera {
    LookAt { from: Vec3, to: Vec3, up: Vec3 },
    Orbital { state: Vec2, distance: f32 },
}

#[derive(Debug)]
pub struct LoadedObject {
    vertex: VertexBuffer<Vertex>,
    index: IndexBuffer<u32>,
    matrix: Mat4,
}

#[derive(Debug)]
pub struct LoadedTarget {
    color: Vec<(usize, String)>,
    depth: Option<usize>,
}

#[derive(Debug)]
pub struct LoadedPasses {
    vertex: Shader,
    fragment: Shader,
    program: Program,
    draw_parameters: DrawParameters<'static>,
    objects: Vec<usize>,
    textures: Vec<(usize, String)>,
    target: Option<LoadedTarget>,
}

#[derive(Debug)]
pub struct Config {
    mouse_pressed: bool,
    config: ser::Config,
    camera: LoadedCamera,
    objects: Vec<LoadedObject>,
    textures: Vec<LoadedTexture>,
    passes: Vec<LoadedPasses>,
    display: Display,
}

impl Config {
    pub fn load(path: impl AsRef<Path>, display: &Display) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path).context("could not find config file")?;
        let config: ser::Config = match path.extension().and_then(OsStr::to_str) {
            Some("ron") => ron::de::from_reader(file).context("Failed to parse config file")?,
            Some("json") => serde_json::from_reader(file).context("Failed to parse config file")?,
            _ => bail!("Invalid config extension!"),
        };

        let mut object_name_match = HashMap::new();

        let objects = config
            .objects
            .iter()
            .enumerate()
            .try_fold::<_, _, Result<_>>(Vec::new(), |mut acc, (idx, x)| {
                object_name_match.insert(x.name.clone(), idx);
                acc.push(Self::load_object(x, display)?);
                Result::Ok(acc)
            })?;

        let mut texture_name_match = HashMap::new();

        let textures = config
            .textures
            .iter()
            .enumerate()
            .try_fold::<_, _, Result<_>>(Vec::new(), |mut acc, (idx, x)| {
                texture_name_match.insert(x.name.clone(), idx);
                acc.push(LoadedTexture::load(x, display)?);
                Result::Ok(acc)
            })?;

        let passes = config
            .passes
            .iter()
            .enumerate()
            .try_fold::<_, _, Result<_>>(Vec::new(), |mut acc, (idx, x)| {
                acc.push(Self::load_pass(
                    x,
                    &object_name_match,
                    &texture_name_match,
                    display,
                    idx,
                )?);
                Result::Ok(acc)
            })?;

        let camera = match config.camera.kind {
            CameraKind::Lookat { from, to, up } => LoadedCamera::LookAt { from, to, up },
            CameraKind::Orbital { distance, .. } => LoadedCamera::Orbital {
                state: Vec2::ZERO,
                distance,
            },
        };

        debug!("reloaded config: {:#?}", &config);

        Ok(Config {
            mouse_pressed: false,
            config,
            objects,
            textures,
            passes,
            display: display.clone(),
            camera,
        })
    }

    fn link_texture(
        texture: &ser::TextureRef,
        texture_name_match: &HashMap<String, usize>,
        pass_num: usize,
    ) -> Result<(usize, String)> {
        match texture {
            ser::TextureRef::Renamed { name, r#as } => {
                if let Some(x) = texture_name_match.get(name).copied() {
                    return Ok((x, r#as.clone()));
                } else {
                    let mut expects = String::new();
                    write!(expects, "Expected one of ").unwrap();
                    for (idx, k) in texture_name_match.keys().enumerate() {
                        if idx != 0 {
                            write!(expects, ",").unwrap();
                        }
                        write!(expects, "`{}`", k).unwrap();
                    }
                    write!(expects, ".").unwrap();

                    bail!(
                        "Could not find texture `{}` for pass {}. {}",
                        name,
                        pass_num,
                        expects
                    )
                }
            }
            ser::TextureRef::Name(name) => {
                if let Some(x) = texture_name_match.get(name).copied() {
                    return Ok((x, name.clone()));
                } else {
                    let mut expects = String::new();
                    write!(expects, "Expected one of ").unwrap();
                    for (idx, k) in texture_name_match.keys().enumerate() {
                        if idx != 0 {
                            write!(expects, ",").unwrap();
                        }
                        write!(expects, "`{}`", k).unwrap();
                    }
                    write!(expects, ".").unwrap();

                    bail!(
                        "Could not find texture `{}` for pass {}. {}",
                        name,
                        pass_num,
                        expects
                    )
                }
            }
        };
    }

    pub fn load_pass(
        pass: &ser::Pass,
        object_name_match: &HashMap<String, usize>,
        texture_name_match: &HashMap<String, usize>,
        display: &Display,
        pass_num: usize,
    ) -> Result<LoadedPasses> {
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

                bail!(
                    "Could not find object `{}` for pass {}. {}",
                    x,
                    pass_num,
                    expects
                )
            }
            Ok(acc)
        })?;

        let textures =
            pass.textures
                .iter()
                .try_fold::<_, _, Result<_>>(Vec::new(), |mut acc, x| {
                    acc.push(
                        Self::link_texture(x, texture_name_match, pass_num)
                            .context("failed to link pass texture")?,
                    );
                    Result::Ok(acc)
                })?;

        let vertex = Shader::load(&pass.vertex_shader)
            .with_context(|| format!("Failed to load vertex shader for pass: {}", pass_num))?;
        let fragment = Shader::load(&pass.fragment_shader)
            .with_context(|| format!("Failed to load fragment shader for pass: {}", pass_num))?;

        let program = Program::from_source(display, &vertex.source, &fragment.source, None)
            .with_context(|| format!("Failed to compile program for pass: {}", pass_num))?;

        for (name, _) in program.attributes() {
            match name.as_str() {
                "position" | "normal" | "tex_coord" => {}
                x => bail!(
                    "Invalid attribute `{}` used in shader for pass {}",
                    x,
                    pass_num
                ),
            }
        }

        let target = match pass.target {
            ser::PassTarget::Frame => None,
            ser::PassTarget::Buffer(ref x) => {
                let color = x
                    .color
                    .iter()
                    .try_fold::<_, _, Result<_>>(Vec::new(), |mut acc, x| {
                        acc.push(Self::link_texture(&x, texture_name_match, pass_num)?);
                        Ok(acc)
                    })
                    .context("Failed to link pass target color attachment")?;
                let depth = x
                    .depth
                    .as_ref()
                    .map(|x| {
                        Self::link_texture(
                            &ser::TextureRef::Name(x.clone()),
                            texture_name_match,
                            pass_num,
                        )
                    })
                    .transpose()
                    .context("Failed to link pass target depth attachment")?
                    .map(|x| x.0);

                Some(LoadedTarget { color, depth })
            }
        };

        let draw_parameters = pass.settings.to_params();

        Ok(LoadedPasses {
            vertex,
            fragment,
            objects,
            draw_parameters,
            textures,
            program,
            target,
        })
    }

    pub fn load_object(object: &ser::Object, display: &Display) -> Result<LoadedObject> {
        let rot = Quat::from_rotation_ypr(
            object.rotation.x.to_radians(),
            object.rotation.y.to_radians(),
            object.rotation.z.to_radians(),
        );
        let mat = Mat4::from_quat(rot)
            * Mat4::from_scale(object.scale)
            * Mat4::from_translation(object.position);
        let geom = match object.kind {
            ser::ObjectKind::Geometry(ref x) => x
                .to_buffers(display)
                .context("Failed to load model geometry")?,
        };
        Ok(LoadedObject {
            matrix: mat,
            vertex: geom.0,
            index: geom.1,
        })
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => match state {
                ElementState::Pressed => {
                    self.display.gl_window().window().set_cursor_grab(true).ok();
                    self.display.gl_window().window().set_cursor_visible(false);
                    self.mouse_pressed = true;
                }
                ElementState::Released => {
                    self.display
                        .gl_window()
                        .window()
                        .set_cursor_grab(false)
                        .ok();
                    self.display.gl_window().window().set_cursor_visible(true);
                    self.mouse_pressed = false;
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let delta = match delta {
                    MouseScrollDelta::LineDelta(_, x) => *x,
                    MouseScrollDelta::PixelDelta(x) => x.y as f32 * 20.0,
                };

                match &mut self.camera {
                    LoadedCamera::Orbital {
                        ref mut distance, ..
                    } => {
                        self.display.gl_window().window().request_redraw();
                        *distance = 0.0f32.max(*distance + delta);
                    }
                    //ser::CameraKind::Flying { mut speed } => speed += delta,
                    _ => {}
                }
            }
            WindowEvent::Resized(size) => {
                let dimensions = (size.width, size.height);
                for t in self.textures.iter_mut() {
                    t.resize(dimensions, &self.display).unwrap()
                }
            }
            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => match &mut self.camera {
                LoadedCamera::Orbital { ref mut state, .. } => {
                    if self.mouse_pressed {
                        self.display.gl_window().window().request_redraw();
                        *state += Vec2::new(delta.0 as f32, -delta.1 as f32);
                    }
                }
                LoadedCamera::LookAt { .. } => {}
            },
            _ => {}
        }
    }
}
