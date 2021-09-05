use std::path::Path;

use super::ser;
use crate::asset::{Asset, AssetRef, DynAssetRef};
use anyhow::{Context, Result};
use glium::{
    texture::{RawImage2d, Texture2d},
    Display,
};
use image::RgbaImage;

#[derive(Debug)]
pub enum LoadedTextureKind {
    File(AssetRef<FileTexture>),
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
        };
        Ok(LoadedTexture {
            kind,
            texture,
            config: config.clone(),
        })
    }

    pub fn try_reload(&mut self, asset: &DynAssetRef, display: &Display) -> Result<bool> {
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
        }
        Ok(false)
    }
}
