use crate::{
    app::{Resource, ResourceId},
    config::Pass as ConfigPass,
};
use anyhow::Result;
use glium::{Display, Program};
use std::{fs::File, io::Read};

pub struct Shader {
    source: String,
    generation: u32,
}

impl Resource for Shader {
    fn load(file: File, display: &Display) -> Result<Self> {
        let mut source = String::new();
        file.write_to_string(&mut source)?;
        Ok(Shader {
            source,
            generation: 0,
        })
    }

    fn reload(&mut self, file: File, display: &Display) -> Result<()> {
        let mut source = String::new();
        file.write_to_string(&mut source)?;
        self.source = source;
        self.generation += 1;
    }
}

pub struct Pass {
    vertex_shader: ResourceId<Shader>,
    fragment_shader: ResourceId<Shader>,
    program: Program,
}

impl Pass {
    pub fn new(pass: ConfigPass, res: &Resources) -> Result<Self> {}
}
