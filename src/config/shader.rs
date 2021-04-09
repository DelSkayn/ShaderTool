use shaderc::{ShaderKind, CompilationArtifact};
use spirv_reflect::ShaderModule;
use crate::{
    resources::{Resource, Resources},
    State,
};
use std::{
    path::Path,
    fs::File,
    io::Read,
};
use anyhow::{Context,Result, anyhow};

pub struct Shader{
    kind: ShaderKind,
    source: String,
    spirv: CompilationArtifact,
    reflect: ShaderModule,
}

impl Resource for Shader{
    type Context = ShaderKind;

    fn load(path: &Path, ctx: Self::Context, state: &mut State, _res: &mut Resources) -> Result<Self> {
        let mut source = String::new();
        let mut file = File::open(&path)
            .context("failed to open shader file")?;
        file.read_to_string(&mut source)
            .context("failed to read shader file")?;

        let spirv = state.compiler.compile_into_spirv(
            &source,
            ctx,
            &format!("{}",path.display()),
            "main",
            None)
            .context("failed to compile shader")?;

        let reflect = spirv_reflect::ShaderModule::load_u32_data(spirv.as_binary())
            .map_err(|e| anyhow!("{}",e))
            .context("failed to analyze shader")?;

        Ok(Shader{
            kind: ctx,
            source,
            spirv,
            reflect
        })
    }

    fn reload(&mut self, path: &Path, state: &mut State, res: &mut Resources) -> Result<()> {
        *self = Self::load(path,self.kind,state,res)?;
        Ok(())
    }

}
