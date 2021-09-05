use super::{Config, LoadedCamera};
use anyhow::Result;
use glam::f32::{Mat4, Quat, Vec3};
use std::collections::HashMap;

use glium::{
    uniforms::{AsUniformValue, Sampler, UniformValue, Uniforms, UniformsStorage},
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

    pub fn render(&self, frame: &mut Frame) -> Result<()> {
        let camera_mat = self.get_camera_matrix();
        let dimensions = frame.get_dimensions();
        let aspect_ratio = dimensions.0 as f32 / dimensions.1 as f32;
        let perspective_mat = Mat4::perspective_lh(
            self.config.camera.fov.to_radians(),
            aspect_ratio,
            0.01,
            100.0,
        );

        for pass in self.passes.iter() {
            let mut uniforms = DynUniformStorage::new();
            let camera_mat = camera_mat.to_cols_array_2d();
            let perspective_mat = perspective_mat.to_cols_array_2d();

            let samplers: Vec<_> = pass
                .textures
                .iter()
                .map(|(text_id, name)| {
                    let texture = &self.textures[*text_id];
                    let sampler = Sampler::new(&texture.texture);
                    let sampler = texture.config.apply_to_sampler(sampler);
                    (name, sampler)
                })
                .collect();

            uniforms.add("view".to_string(), &camera_mat);
            uniforms.add("projection".to_string(), &perspective_mat);

            for (name, s) in samplers.iter() {
                uniforms.add(format!("texture_{}", name), s)
            }

            for object in pass.objects.iter().copied() {
                let object = &self.objects[object];
                let model = object.matrix.to_cols_array_2d();
                let mut uniforms = uniforms.clone();
                uniforms.add("model".to_string(), &model);

                frame.draw(
                    &object.vertex,
                    &object.index,
                    &pass.program,
                    &uniforms,
                    &pass.draw_parameters,
                )?;
            }
        }
        Ok(())
    }
}
