use serde::{Deserialize, Serialize};

use glium::draw_parameters::{
    self as dp, BackfaceCullingMode, DepthClamp, DepthTest, DrawParameters,
};

#[derive(Serialize, Deserialize)]
#[serde(remote = "DepthClamp")]
#[serde(rename_all = "snake_case")]
pub enum DepthClampDef {
    NoClamp,
    Clamp,
    ClampNear,
    ClampFar,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(remote = "DepthTest")]
pub enum DepthTestDef {
    #[serde(rename = "ignore")]
    Ignore,
    #[serde(rename = "overwrite")]
    Overwrite,
    #[serde(rename = "equal")]
    IfEqual,
    #[serde(rename = "not_equal")]
    IfNotEqual,
    #[serde(rename = "greater")]
    IfMore,
    #[serde(rename = "greater_equal")]
    IfMoreOrEqual,
    #[serde(rename = "less")]
    IfLess,
    #[serde(rename = "less_equal")]
    IfLessOrEqual,
}

fn t() -> bool {
    true
}

fn clamp() -> DepthClamp {
    DepthClamp::NoClamp
}

fn test() -> DepthTest {
    DepthTest::IfLess
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Depth {
    #[serde(default = "test")]
    #[serde(with = "DepthTestDef")]
    compare: DepthTest,
    #[serde(default = "t")]
    write: bool,
    #[serde(default = "clamp")]
    #[serde(with = "DepthClampDef")]
    clamp: DepthClamp,
}

impl Default for Depth {
    fn default() -> Self {
        Depth {
            compare: DepthTest::IfLess,
            write: true,
            clamp: DepthClamp::NoClamp,
        }
    }
}

fn cull() -> BackfaceCullingMode {
    BackfaceCullingMode::CullCounterClockwise
}

#[derive(Deserialize, Serialize)]
#[serde(remote = "BackfaceCullingMode")]
pub enum BackfaceCullingModeDef {
    #[serde(rename = "disabled")]
    CullingDisabled,
    #[serde(rename = "counter_clockwise")]
    CullCounterClockwise,
    #[serde(rename = "clockwise")]
    CullClockwise,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Settings {
    #[serde(default)]
    depth: Depth,
    #[serde(with = "BackfaceCullingModeDef")]
    #[serde(default = "cull")]
    cull: BackfaceCullingMode,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            depth: Depth::default(),
            cull: cull(),
        }
    }
}

impl Settings {
    pub fn to_params(&self) -> DrawParameters<'static> {
        DrawParameters {
            depth: dp::Depth {
                test: self.depth.compare,
                write: self.depth.write,
                clamp: self.depth.clamp,
                ..Default::default()
            },
            backface_culling: self.cull,
            ..DrawParameters::default()
        }
    }
}
