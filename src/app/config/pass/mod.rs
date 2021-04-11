use crate::app::{resource::AnyResourceId, RenderData, ResourceId, Resources};
use anyhow::{Context, Result};
use glam::{Mat4, Quat};
use glium::{uniform, Display, DrawParameters, Frame, Program, Surface};
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, f32, sync::Arc};

use super::*;

mod settings;
use settings::Settings;

#[derive(Debug, Serialize, Deserialize)]
pub struct Pass {
    vertex_shader: String,
    fragment_shader: String,
    #[serde(default)]
    objects: Vec<String>,
    #[serde(default)]
    settings: Settings,
}

pub struct LoadedPass {
    pub vertex_shader: ResourceId<Shader>,
    pub fragment_shader: ResourceId<Shader>,
    program: Program,
    objects: Vec<Arc<LoadedObject>>,
    draw_parameters: DrawParameters<'static>,
}

impl LoadedPass {
    pub fn new(
        pass: &Pass,
        loaded_objects: &HashMap<String, Arc<LoadedObject>>,
        display: &Display,
        res: &mut Resources,
    ) -> Result<Self> {
        let mut objects = Vec::new();

        for object in pass.objects.iter() {
            if let Some(x) = loaded_objects.get(object) {
                objects.push(x.clone());
            } else {
                anyhow::bail!("Object with name \"{}\" not defined!", object);
            }
        }

        let vertex_shader: ResourceId<Shader> = res
            .insert(&pass.vertex_shader, display)
            .context("Loading vertex shader")?;
        let fragment_shader: ResourceId<Shader> = res
            .insert(&pass.fragment_shader, display)
            .context("Loading fragment shader")?;

        let program = Program::from_source(
            display,
            &res.get(&vertex_shader).unwrap().source,
            &res.get(&fragment_shader).unwrap().source,
            None,
        )
        .context("Compiling shader program")?;
        Ok(LoadedPass {
            vertex_shader,
            fragment_shader,
            program,
            objects,
            draw_parameters: pass.settings.to_params()
        })
    }

    pub fn reload(&mut self, dep: AnyResourceId, display: &Display, res: &Resources) -> Result<()> {
        if self.vertex_shader.into_any() != dep && self.fragment_shader.into_any() != dep {
            return Ok(());
        }

        let program = Program::from_source(
            display,
            &res.get(&self.vertex_shader).unwrap().source,
            &res.get(&self.fragment_shader).unwrap().source,
            None,
        )
        .context("Failed to compile shader program")?;
        self.program = program;
        Ok(())
    }

    pub fn render(&self, frame: &mut Frame, data: &RenderData) -> Result<()> {
        for object in self.objects.iter() {
            let rot = object.rotation / 360.0 * 2.0 * f32::consts::PI;
            let rotation = Quat::from_rotation_ypr(rot.x, rot.y, rot.z);

            let model =
                Mat4::from_scale_rotation_translation(object.scale, rotation, object.position);

            let uniforms = uniform! {
                model: model.to_cols_array_2d(),
                view: data.view.to_cols_array_2d(),
                projection: data.projection.to_cols_array_2d(),
            };

            frame.draw(
                &object.verticies,
                &object.indicies,
                &self.program,
                &uniforms,
                &self.draw_parameters,
            )?;
        }
        Ok(())
    }
}
