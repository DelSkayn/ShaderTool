use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct Object;

#[derive(Deserialize)]
pub struct Pass;

#[derive(Deserialize)]
pub enum Camera {}

#[derive(Deserialize)]
pub struct Config {
    objects: Vec<Object>,
    passes: Vec<Pass>,
    camera: Camera,
}

#[cfg(not(all(feature = "config_ron", feature = "config_json")))]
compile_error!("Neither the ron nor the json feature is enabled. Enable atleast on otherwise there no way to load the config file!");

impl Config {
    // Load the config from the current directory
    pub fn load() -> Result<Self> {
        #[cfg(feature = "config_json")]
        if Path::new("./ShaderTool.json").exists() {
            return Ok(serde_json::from_reader(File::open("./ShaderTool.json")?)?);
        }
        #[cfg(feature = "config_ron")]
        if Path::new("./ShaderTool.ron").exists() {
            return Ok(ron::de::from_reader(File::open("./ShaderTool.ron")?)?);
        }
        bail!("Could not find the ShaderTool config file")
    }
}
