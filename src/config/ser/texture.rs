use glium::{
    texture::{DepthFormat, MipmapsOption, UncompressedFloatFormat},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler, SamplerWrapFunction},
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum TextureSize {
    ViewPort,
    Size { width: u32, height: u32 },
}

impl Default for TextureSize {
    fn default() -> Self {
        TextureSize::ViewPort
    }
}

fn text_format() -> UncompressedFloatFormat {
    UncompressedFloatFormat::F32F32F32F32
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct EmptyTexture {
    pub size: TextureSize,
    #[serde(with = "UncompressedFloatFormatDef")]
    #[serde(default = "text_format")]
    pub format: UncompressedFloatFormat,
}

fn depth_format() -> DepthFormat {
    DepthFormat::F32
}

#[derive(Deserialize, Debug, Clone)]
pub struct DepthTexture {
    pub size: TextureSize,
    #[serde(with = "DepthFormatDef")]
    #[serde(default = "depth_format")]
    pub format: DepthFormat,
}

#[derive(Deserialize, Debug, Clone)]
pub enum TextureKind {
    File(String),
    Empty(EmptyTexture),
    Depth(DepthTexture),
}

fn wrap() -> SamplerWrapFunction {
    SamplerWrapFunction::Repeat
}

fn minify_filter() -> MinifySamplerFilter {
    MinifySamplerFilter::Linear
}

fn magnify_filter() -> MagnifySamplerFilter {
    MagnifySamplerFilter::Linear
}

#[derive(Deserialize, Debug, Clone)]
pub struct Texture {
    pub name: String,
    pub kind: TextureKind,
    #[serde(with = "SamplerWrapFunctionDef")]
    #[serde(default = "wrap")]
    pub wrap: SamplerWrapFunction,
    #[serde(with = "MinifySamplerFilterDef")]
    #[serde(default = "minify_filter")]
    pub minify_filter: MinifySamplerFilter,
    #[serde(with = "MagnifySamplerFilterDef")]
    #[serde(default = "magnify_filter")]
    pub magnify_filter: MagnifySamplerFilter,
    #[serde(default)]
    pub anisotropy: Option<u16>,
    #[serde(default)]
    pub mipmaps: Mipmaps,
}

impl Texture {
    pub fn apply_to_sampler<'t, T>(&self, sampler: Sampler<'t, T>) -> Sampler<'t, T> {
        let res = sampler
            .wrap_function(self.wrap)
            .minify_filter(self.minify_filter)
            .magnify_filter(self.magnify_filter);

        if let Some(x) = self.anisotropy {
            res.anisotropy(x)
        } else {
            res
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(remote = "SamplerWrapFunction")]
pub enum SamplerWrapFunctionDef {
    Repeat,
    Mirror,
    Clamp,
    BorderClamp,
    MirrorClamp,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(remote = "MinifySamplerFilter")]
pub enum MinifySamplerFilterDef {
    Nearest,
    Linear,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(remote = "MagnifySamplerFilter")]
pub enum MagnifySamplerFilterDef {
    Nearest,
    Linear,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Mipmaps {
    None,
    Empty,
    EmptyAmount(u32),
    Generate,
    GenerateAmount(u32),
}

impl From<Mipmaps> for MipmapsOption {
    fn from(m: Mipmaps) -> Self {
        match m {
            Mipmaps::None => MipmapsOption::NoMipmap,
            Mipmaps::Empty => MipmapsOption::EmptyMipmaps,
            Mipmaps::EmptyAmount(x) => MipmapsOption::EmptyMipmapsMax(x),
            Mipmaps::Generate => MipmapsOption::AutoGeneratedMipmaps,
            Mipmaps::GenerateAmount(x) => MipmapsOption::AutoGeneratedMipmapsMax(x),
        }
    }
}

impl Default for Mipmaps {
    fn default() -> Self {
        Mipmaps::None
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(remote = "UncompressedFloatFormat")]
pub enum UncompressedFloatFormatDef {
    U8,
    ///
    ///
    /// Guaranteed to be supported for textures.
    I8,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    U16,
    ///
    ///
    /// Guaranteed to be supported for textures.
    I16,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    U8U8,
    ///
    ///
    /// Guaranteed to be supported for textures.
    I8I8,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    U16U16,
    ///
    ///
    /// Guaranteed to be supported for textures.
    I16I16,
    ///
    U3U3U2,
    ///
    U4U4U4,
    ///
    U5U5U5,
    ///
    ///
    /// Guaranteed to be supported for textures.
    U8U8U8,
    ///
    ///
    /// Guaranteed to be supported for textures.
    I8I8I8,
    ///
    U10U10U10,
    ///
    U12U12U12,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    U16U16U16,
    ///
    ///
    /// Guaranteed to be supported for textures.
    I16I16I16,
    ///
    U2U2U2U2,
    ///
    U4U4U4U4,
    ///
    U5U5U5U1,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    U8U8U8U8,
    ///
    ///
    /// Guaranteed to be supported for textures.
    I8I8I8I8,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    U10U10U10U2,
    ///
    U12U12U12U12,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    U16U16U16U16,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    I16I16I16I16,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    F16,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    F16F16,
    ///
    ///
    /// Guaranteed to be supported for textures.
    F16F16F16,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    F16F16F16F16,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    F32,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    F32F32,
    ///
    ///
    /// Guaranteed to be supported for textures.
    F32F32F32,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    F32F32F32F32,
    ///
    ///
    /// Guaranteed to be supported for both textures and renderbuffers.
    F11F11F10,
    /// Uses three components of 9 bits of precision that all share the same exponent.
    ///
    /// Use this format only if all the components are approximately equal.
    ///
    /// Guaranteed to be supported for textures.
    F9F9F9,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(remote = "DepthFormat")]
pub enum DepthFormatDef {
    I16,
    I24,
    I32,
    F32,
}
