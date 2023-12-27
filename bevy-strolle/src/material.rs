use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use strolle as st;

use crate::utils::{color_to_vec4, GlamCompat};
use crate::EngineParams;

/// Extends Bevy's `StandardMaterial` with extra features supported by Strolle.
#[derive(Asset, Clone, Debug, TypePath, AsBindGroup)]
pub struct StrolleMaterial {
    pub parent: StandardMaterial,

    /// Index of refraction; defaults to 1.0 (air).
    pub ior: f32,
}

impl Default for StrolleMaterial {
    fn default() -> Self {
        Self {
            parent: Default::default(),
            ior: 1.0,
        }
    }
}

impl Material for StrolleMaterial {
    //
}

pub(crate) trait MaterialLike
where
    Self: TypePath + Asset + Clone + Send + Sync + 'static,
{
    fn into_material(self) -> st::Material<EngineParams>;
    fn map_id(handle: AssetId<Self>) -> MaterialId;
    fn images(&self) -> Vec<&Handle<Image>>;
}

impl MaterialLike for StandardMaterial {
    fn into_material(self) -> st::Material<EngineParams> {
        let base_color = {
            let color = color_to_vec4(self.base_color);

            match self.alpha_mode {
                AlphaMode::Opaque => color.xyz().extend(1.0),
                AlphaMode::Mask(mask) => {
                    if color.w >= mask {
                        color.xyz().extend(1.0)
                    } else {
                        color.xyz().extend(0.0)
                    }
                }
                _ => color,
            }
        };

        let alpha_mode = match self.alpha_mode {
            AlphaMode::Opaque => st::AlphaMode::Opaque,
            _ => st::AlphaMode::Blend,
        };

        st::Material {
            base_color: base_color.compat(),
            base_color_texture: self
                .base_color_texture
                .map(|handle| handle.id()),
            emissive: color_to_vec4(self.emissive).compat(),
            emissive_texture: self.emissive_texture.map(|handle| handle.id()),
            perceptual_roughness: self.perceptual_roughness,
            metallic: self.metallic,
            reflectance: self.reflectance,
            normal_map_texture: self
                .normal_map_texture
                .map(|handle| handle.id()),
            alpha_mode,
            ..Default::default()
        }
    }

    fn map_id(id: AssetId<Self>) -> MaterialId {
        MaterialId::Standard(id)
    }

    fn images(&self) -> Vec<&Handle<Image>> {
        self.base_color_texture
            .as_ref()
            .into_iter()
            .chain(self.normal_map_texture.as_ref())
            .collect()
    }
}

impl MaterialLike for StrolleMaterial {
    fn into_material(self) -> st::Material<EngineParams> {
        st::Material {
            ior: self.ior,
            ..self.parent.into_material()
        }
    }

    fn map_id(id: AssetId<Self>) -> MaterialId {
        MaterialId::Strolle(id)
    }

    fn images(&self) -> Vec<&Handle<Image>> {
        self.parent.images()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MaterialId {
    Standard(AssetId<StandardMaterial>),
    Strolle(AssetId<StrolleMaterial>),
}
