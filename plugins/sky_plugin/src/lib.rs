use bevy::{
    math::Vec3,
    prelude::{
        AddAsset, App, Bundle, Color, Draw, GlobalTransform, Handle, Mesh, Plugin, RenderPipelines,
        Transform, Visible,
    },
    reflect::TypeUuid,
    render::{pipeline::RenderPipeline, render_graph::base::MainPass},
};
use bevy_inspector_egui::{Inspectable, InspectableRegistry};
use pipeline::{add_sky_pipeline, SkyMaterial};
use systems::{rotate_sky, update_sky_materials};

use crate::pipeline::SKY_PIPELINE_HANDLE;

pub mod cube_sphere;
pub mod pipeline;
mod systems;

pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<SkyMaterial>()
            .add_system(rotate_sky)
            .add_system(update_sky_materials);

        add_sky_pipeline(&mut app.world);

        let mut registry = app
            .world
            .get_resource_or_insert_with(InspectableRegistry::default);

        registry.register::<SkyMaterialData>();
    }
}

/// Stores all parameters for the sky material.
/// It is adjustable via the inspector.
#[derive(Inspectable, TypeUuid)]
#[uuid = "1b1650ae-6690-4bd4-a0c3-88a947b88a3d"]
pub struct SkyMaterialData {
    sky_color: Color,
    horizon_color: Color,
    star_count: f32,
    star_sharpness: f32,
    sun_color: Color,
    sun_size: f32,
    moon_color: Color,
    moon_size: f32,
}

impl Default for SkyMaterialData {
    fn default() -> Self {
        Self {
            sky_color: Color::rgb(0.0, 0.0, 1.0),
            horizon_color: Color::rgb(0.0, 0.5, 1.0),
            star_count: 50.0,
            star_sharpness: 100.0,
            sun_color: Color::rgb(1.0, 1.0, 0.0),
            sun_size: 0.2,
            moon_color: Color::rgb(0.9, 0.9, 0.9),
            moon_size: 0.1,
        }
    }
}

/// A bundle containing all the components required to spawn the water.
#[derive(Bundle)]
pub struct SkyBundle {
    pub sky: Sky,
    pub mesh: Handle<Mesh>,
    pub material: Handle<SkyMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for SkyBundle {
    fn default() -> Self {
        Self {
            sky: Default::default(),
            mesh: Default::default(),
            material: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            visible: Default::default(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                SKY_PIPELINE_HANDLE.typed(),
            )]),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct Sky {
    pub radius: f32,
    pub rotation_axis: Vec3,
    pub period: f32,
    pub cycle: f32,
}

pub struct Sun {
    pub rotation_axis: Vec3,
    pub period: f32,
    pub cycle: f32,
}

pub struct Moon;
