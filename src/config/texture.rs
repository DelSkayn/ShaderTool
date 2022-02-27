use std::path::Path;

use super::ser::{self, TextureSize};
use anyhow::{Context, Result};
use glium::{
    texture::{DepthFormat, DepthTexture2d, RawImage2d, Texture2d, UncompressedFloatFormat},
    Display,
};
use image::RgbaImage;

#[derive(Debug)]
pub enum LoadedTextureKind {
    File {
        texture: Texture2d,
    },
    Empty {
        size: TextureSize,
        format: UncompressedFloatFormat,
        texture: Texture2d,
    },
    Depth {
        size: TextureSize,
        format: DepthFormat,
        texture: DepthTexture2d,
    },
}

#[derive(Debug)]
pub struct FileTexture {
    image: RgbaImage,
}

impl FileTexture {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        Ok(FileTexture {
            image: image::open(path)?.to_rgba8(),
        })
    }
}

#[derive(Debug)]
pub struct LoadedTexture {
    pub kind: LoadedTextureKind,
    pub config: ser::Texture,
}

impl LoadedTexture {
    /// Load a texture from a config.
    pub fn load(config: &ser::Texture, display: &Display) -> Result<Self> {
        let kind = match config.kind {
            ser::TextureKind::File(ref x) => {
                let loaded = FileTexture::load(x).with_context(|| {
                    format!("failed to load image file for texture at path: {}", x)
                })?;
                let dimensions = loaded.image.dimensions();
                let raw_image = RawImage2d::from_raw_rgba(loaded.image.into_vec(), dimensions);
                let texture = Texture2d::with_mipmaps(display, raw_image, config.mipmaps.into())
                    .context("failed to load texture")?;
                LoadedTextureKind::File { texture }
            }
            ser::TextureKind::Empty(ref x) => {
                let size = match x.size {
                    TextureSize::ViewPort => display.get_framebuffer_dimensions(),
                    TextureSize::Size { width, height } => (width, height),
                };
                let texture = Texture2d::empty_with_format(
                    display,
                    x.format,
                    config.mipmaps.into(),
                    size.0,
                    size.1,
                )
                .context("failed to create texture")?;
                LoadedTextureKind::Empty {
                    size: x.size,
                    format: x.format,
                    texture,
                }
            }
            ser::TextureKind::Depth(ref x) => {
                let size = match x.size {
                    TextureSize::ViewPort => display.get_framebuffer_dimensions(),
                    TextureSize::Size { width, height } => (width, height),
                };
                let texture = DepthTexture2d::empty_with_format(
                    display,
                    x.format,
                    config.mipmaps.into(),
                    size.0,
                    size.1,
                )
                .context("failed to create texture")?;
                LoadedTextureKind::Depth {
                    size: x.size,
                    format: x.format,
                    texture,
                }
            }
        };
        Ok(LoadedTexture {
            kind,
            config: config.clone(),
        })
    }

    /// Resizes the texture if the texture size is a factor of the viewport size.
    pub fn resize(&mut self, dimensions: (u32, u32), display: &Display) -> Result<()> {
        match self.kind {
            LoadedTextureKind::File { .. } => {}
            LoadedTextureKind::Empty {
                size,
                format,
                ref mut texture,
            } => match size {
                TextureSize::Size { .. } => {}
                TextureSize::ViewPort => {
                    *texture = Texture2d::empty_with_format(
                        display,
                        format,
                        self.config.mipmaps.into(),
                        dimensions.0,
                        dimensions.1,
                    )
                    .context("failed to create texture")?;
                }
            },
            LoadedTextureKind::Depth {
                size,
                format,
                ref mut texture,
            } => match size {
                TextureSize::Size { .. } => {}
                TextureSize::ViewPort => {
                    *texture = DepthTexture2d::empty_with_format(
                        display,
                        format,
                        self.config.mipmaps.into(),
                        dimensions.0,
                        dimensions.1,
                    )
                    .context("failed to create texture")?;
                }
            },
        }
        Ok(())
    }
}
