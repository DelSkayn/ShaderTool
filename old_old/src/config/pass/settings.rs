use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DepthClamp {
    NoClamp,
    Clamp,
    ClampNear,
    ClampFar,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DepthTest {
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
    compare: DepthTest,
    #[serde(default = "t")]
    write: bool,
    #[serde(default = "clamp")]
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

#[derive(Debug, Deserialize, Serialize)]
pub enum BackfaceCullingMode {
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
