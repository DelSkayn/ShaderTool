use std::path::Path;

use super::ser::{self, TextureSize};
use crate::asset::{Asset, AssetRef, DynAssetRef};
use anyhow::{Context, Result};
use glium::{
    texture::{RawImage2d, Texture2d, UncompressedFloatFormat},
    Display,
};
use image::RgbaImage;

#[derive(Debug)]
pub enum LoadedTextureKind {
    File(AssetRef<FileTexture>),
    Empty {
        size: TextureSize,
        format: UncompressedFloatFormat,
    },
}

#[derive(Debug)]
pub struct FileTexture {
    image: RgbaImage,
}

impl FileTexture {
    fn load(path: &Path, _args: ()) -> Result<Self> {
        Ok(FileTexture {
            image: image::open(path)?.to_rgba8(),
        })
    }
}

impl Asset for FileTexture {
    fn reload(&mut self, path: &std::path::Path) -> Result<()> {
        *self = Self::load(path, ())?;
        Ok(())
    }

    fn reload_dependency(&mut self, _asset: &crate::asset::DynAssetRef) -> Result<bool> {
        Ok(false)
    }
}

#[derive(Debug)]
pub struct LoadedTexture {
    pub kind: LoadedTextureKind,
    pub texture: Texture2d,
    pub config: ser::Texture,
}

impl LoadedTexture {
    /// Load a texture from a config.
    pub fn load(config: &ser::Texture, display: &Display) -> Result<Self> {
        let (kind, texture) = match config.kind {
            ser::TextureKind::File(ref x) => {
                let loaded = AssetRef::build(FileTexture::load, x, ()).with_context(|| {
                    format!("failed to load image file for texture at path: {}", x)
                })?;
                let image = loaded.borrow_mut().image.clone();
                let dimensions = image.dimensions();
                let raw_image = RawImage2d::from_raw_rgba(image.into_vec(), dimensions);
                let texture = Texture2d::with_mipmaps(display, raw_image, config.mipmaps.into())
                    .context("failed to load texture")?;
                (LoadedTextureKind::File(loaded), texture)
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
                (
                    LoadedTextureKind::Empty {
                        size: x.size,
                        format: x.format,
                    },
                    texture,
                )
            }
        };
        Ok(LoadedTexture {
            kind,
            texture,
            config: config.clone(),
        })
    }

    /// Reloads a depended texture if present.
    ///
    /// Returns true if a dependecy was reloaded.
    pub fn try_reload_dependecy(&mut self, asset: &DynAssetRef, display: &Display) -> Result<bool> {
        match self.kind {
            LoadedTextureKind::File(ref x) => {
                if asset.same(x) {
                    let image = x.borrow_mut().image.clone();
                    let dimensions = image.dimensions();
                    let raw_image = RawImage2d::from_raw_rgba(image.into_vec(), dimensions);
                    self.texture =
                        Texture2d::with_mipmaps(display, raw_image, self.config.mipmaps.into())
                            .context("failed to load texture")?;
                    return Ok(true);
                }
            }
            LoadedTextureKind::Empty { .. } => {}
        }
        Ok(false)
    }

    /// Resizes the texture if the texture size is a factor of the viewport size.
    pub fn resize(&mut self, dimensions: (u32, u32), display: &Display) -> Result<()> {
        match self.kind {
            LoadedTextureKind::File(_) => {}
            LoadedTextureKind::Empty { size, format } => match size {
                TextureSize::Size { .. } => {}
                TextureSize::ViewPort => {
                    self.texture = Texture2d::empty_with_format(
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
