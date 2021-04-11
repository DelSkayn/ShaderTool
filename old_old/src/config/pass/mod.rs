use crate::{
    config::{LoadedObject, Shader},
    resources::{AnyResourceId, ResourceId, Resources},
    State,
};
use shaderc::ShaderKind;
use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use wgpu::RenderPipeline;

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
    vertex_shader: ResourceId<Shader>,
    index_shader: ResourceId<Shader>,
    pipeline: RenderPipeline,
    objects: Vec<Arc<LoadedObject>>,
}

impl LoadedPass {
    pub fn new(
        pass: &Pass,
        state: &mut State,
        _objects: &HashMap<String, Arc<LoadedObject>>,
        res: &mut Resources,
    ) -> Result<Self> {
        let _vertex_shader = res.insert::<Shader,_>(&pass.vertex_shader,ShaderKind::Vertex,state)?;
        let _fragment_shader = res.insert::<Shader,_>(&pass.fragment_shader,ShaderKind::Fragment,state)?;

        todo!()
    }

    pub fn reload(
        &mut self,
        _dep: AnyResourceId,
        _state: &mut State,
        _res: &Resources,
    ) -> Result<()> {
        todo!()
    }
}
