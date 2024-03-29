use super::{
    texture::LoadedTextureKind, BuiltinUniform, Config, LoadedCamera, LoadedPass, LoadedTarget,
    UniformBinding,
};
use anyhow::{Context, Result};
use glam::f32::{Mat4, Quat, Vec3};
use std::collections::HashMap;

use glium::{
    framebuffer::MultiOutputFrameBuffer,
    uniforms::{AsUniformValue, Sampler, UniformValue, Uniforms},
    Frame, Surface,
};

#[derive(Clone)]
struct DynUniformStorage<'a>(HashMap<String, UniformValue<'a>>);

impl<'a> DynUniformStorage<'a> {
    pub fn new() -> Self {
        DynUniformStorage(HashMap::new())
    }

    pub fn add<U: AsUniformValue + 'a>(&mut self, name: String, u: &'a U) {
        self.0.insert(name, u.as_uniform_value());
    }
}

impl<'b> Uniforms for DynUniformStorage<'b> {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
        for (k, v) in self.0.iter() {
            f(&k, *v)
        }
    }
}

pub struct BuiltinUniforms {
    time: f32,
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    perspective: [[f32; 4]; 4],
    mouse_x: f32,
    mouse_y: f32,
    mouse_pos: [f32; 2],
    window_width: f32,
    window_height: f32,
    window_size: [f32; 2],
}

impl Config {
    pub fn get_camera_matrix(&self) -> Mat4 {
        match self.camera {
            LoadedCamera::LookAt { from, to, up } => Mat4::look_at_lh(from, to, up),
            LoadedCamera::Orbital { state, distance } => {
                let rotation_y = Quat::from_rotation_y(state.x * 0.01);
                let rotation_x = Quat::from_axis_angle(rotation_y * Vec3::X, -state.y * 0.01);
                let rotation = (rotation_x * rotation_y).normalize();
                let position = rotation * Vec3::new(0.0, 0.0, -1.0) * distance;

                Mat4::from_quat(rotation.conjugate()) * Mat4::from_translation(-position)
            }
        }
    }

    pub fn get_builtin_uniforms(&self) -> BuiltinUniforms {
        let perspective = Mat4::perspective_lh(
            self.config.camera.fov.to_radians(),
            self.window_size.x / self.window_size.y,
            0.01,
            100.0,
        )
        .to_cols_array_2d();
        BuiltinUniforms {
            time: self.start_time.elapsed().as_secs_f32(),
            model: Mat4::IDENTITY.to_cols_array_2d(),
            view: self.get_camera_matrix().to_cols_array_2d(),
            perspective,
            mouse_x: self.mouse_pos.x,
            mouse_y: self.mouse_pos.y,
            mouse_pos: self.mouse_pos.into(),
            window_width: self.window_size.x,
            window_height: self.window_size.y,
            window_size: self.window_size.into(),
        }
    }

    pub fn get_target<'a>(
        &'a self,
        pass_id: usize,
        pass: &'a LoadedPass,
        target: &'a LoadedTarget,
    ) -> Result<MultiOutputFrameBuffer<'a>> {
        let targets = target
            .color
            .iter()
            .try_fold(Vec::new(), |mut acc, text| {
                acc.push(match self.textures[text.0].kind {
                    LoadedTextureKind::File { ref texture, .. } => {
                        if pass
                            .program
                            .get_frag_data_location(text.1.as_str())
                            .is_none()
                        {
                            bail!("Pass does not have fragment output `{}`", text.1.as_str())
                        }
                        (text.1.as_str(), texture)
                    }
                    LoadedTextureKind::Empty { ref texture, .. } => {
                        if pass
                            .program
                            .get_frag_data_location(text.1.as_str())
                            .is_none()
                        {
                            bail!("Pass does not have fragment output `{}`", text.1.as_str())
                        }
                        (text.1.as_str(), texture)
                    }
                    LoadedTextureKind::Depth { .. } => {
                        bail!("Tried to use depth texture as color attachment")
                    }
                });
                Ok(acc)
            })
            .with_context(|| format!("Could not render pass {}", pass_id))?;

        match target.depth {
            Some(depth) => {
                let depth_texture = match self.textures[depth].kind {
                    LoadedTextureKind::Depth { ref texture, .. } => texture,
                    _ => bail!("Tried to use color texture as a depth attachment"),
                };

                MultiOutputFrameBuffer::with_depth_buffer(
                    &self.display,
                    targets.into_iter(),
                    depth_texture,
                )
                .context("could not create frame buffer")
                .with_context(|| format!("Could not render pass {}", pass_id))
            }
            None => MultiOutputFrameBuffer::new(&self.display, targets.into_iter())
                .context("could not create frame buffer")
                .with_context(|| format!("Could not render pass {}", pass_id)),
        }
    }

    pub fn render(&self, frame: &mut Frame) -> Result<bool> {
        let mut builtin_uniforms = self.get_builtin_uniforms();

        let mut should_poll = false;

        for (pass_id, pass) in self.passes.iter().enumerate() {
            if let Some(x) = &pass.target {
                let clear_color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
                self.get_target(pass_id, pass, &x)
                    .with_context(|| {
                        format!("Failed to create traget for render pass {}", pass_id)
                    })?
                    .clear_color_and_depth(
                        (
                            clear_color[0],
                            clear_color[1],
                            clear_color[2],
                            clear_color[3],
                        ),
                        1.0,
                    );
            }
            let mut texture_samplers = Vec::new();
            let mut depth_texture_samplers = Vec::new();

            for (text_id, name) in pass.textures.iter() {
                match self.textures[*text_id].kind {
                    LoadedTextureKind::File { ref texture, .. }
                    | LoadedTextureKind::Empty { ref texture, .. } => {
                        let sampler = Sampler::new(texture);
                        let sampler = self.textures[*text_id].config.apply_to_sampler(sampler);
                        texture_samplers.push((name, sampler));
                    }
                    LoadedTextureKind::Depth { ref texture, .. } => {
                        let sampler = Sampler::new(texture);
                        let sampler = self.textures[*text_id].config.apply_to_sampler(sampler);
                        depth_texture_samplers.push((name, sampler));
                    }
                };
            }

            for object in pass.objects.iter().copied() {
                let object = &self.objects[object];
                builtin_uniforms.model = object.matrix.to_cols_array_2d();

                let mut uniforms = DynUniformStorage::new();

                for (name, value) in pass.uniforms.iter() {
                    match value.binding {
                        UniformBinding::Unbound => {}
                        UniformBinding::Custom(ref x) => {
                            uniforms.add(name.clone(), x);
                        }
                        UniformBinding::Builtin(builtin) => match builtin {
                            BuiltinUniform::View => {
                                uniforms.add(name.clone(), &builtin_uniforms.view)
                            }
                            BuiltinUniform::Model => {
                                uniforms.add(name.clone(), &builtin_uniforms.model)
                            }
                            BuiltinUniform::Perspective => {
                                uniforms.add(name.clone(), &builtin_uniforms.perspective)
                            }
                            BuiltinUniform::Time => {
                                should_poll = true;
                                uniforms.add(name.clone(), &builtin_uniforms.time)
                            }
                            BuiltinUniform::MouseX => {
                                uniforms.add(name.clone(), &builtin_uniforms.mouse_x)
                            }
                            BuiltinUniform::MouseY => {
                                uniforms.add(name.clone(), &builtin_uniforms.mouse_y)
                            }
                            BuiltinUniform::MousePos => {
                                uniforms.add(name.clone(), &builtin_uniforms.mouse_pos)
                            }
                            BuiltinUniform::WindowWidth => {
                                uniforms.add(name.clone(), &builtin_uniforms.window_width)
                            }
                            BuiltinUniform::WindowHeight => {
                                uniforms.add(name.clone(), &builtin_uniforms.window_height)
                            }
                            BuiltinUniform::WindowSize => {
                                uniforms.add(name.clone(), &builtin_uniforms.window_size)
                            }
                        },
                    }
                }
                for (name, s) in texture_samplers.iter() {
                    uniforms.add(format!("texture_{}", name), s)
                }

                for (name, s) in depth_texture_samplers.iter() {
                    uniforms.add(format!("texture_{}", name), s)
                }

                match pass.target {
                    None => {
                        frame
                            .draw(
                                &object.vertex,
                                &object.index,
                                &pass.program,
                                &uniforms,
                                &pass.draw_parameters,
                            )
                            .with_context(|| format!("Could not render pass {}", pass_id))?;
                    }
                    Some(ref target) => {
                        let mut target =
                            self.get_target(pass_id, pass, target).with_context(|| {
                                format!("Failed to create traget for render pass {}", pass_id)
                            })?;
                        target
                            .draw(
                                &object.vertex,
                                &object.index,
                                &pass.program,
                                &uniforms,
                                &pass.draw_parameters,
                            )
                            .with_context(|| format!("Could not render pass {}", pass_id))?
                    }
                }
            }
        }
        Ok(should_poll)
    }
}
