use crate::render::preparation_pipeline::TerrainComputePipeline;
use crate::render::terrain_data::GpuTerrainData;
use crate::terrain::TerrainConfig;
use crate::TerrainData;
use bevy::core::Pod;
use bevy::core::Zeroable;
use bevy::ecs::query::QueryItem;
use bevy::ecs::system::lifetimeless::Write;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_component::ExtractComponent;
use bevy::render::render_resource::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use std::mem;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Zeroable, Pod)]
pub(crate) struct NodeUpdate {
    pub(crate) node_id: u32,
    pub(crate) atlas_index: u32, // u16 not supported by std 140
}

#[derive(Clone, Default, Component)]
pub struct QuadtreeUpdate(pub(crate) Vec<NodeUpdate>);

impl ExtractComponent for QuadtreeUpdate {
    type Query = Write<QuadtreeUpdate>;
    type Filter = ();

    fn extract_component(mut item: QueryItem<Self::Query>) -> Self {
        Self(mem::take(&mut item.0))
    }
}

#[derive(Component)]
pub struct GpuQuadtreeUpdate(pub(crate) Vec<(u32, BindGroup)>);

pub(crate) fn prepare_quadtree_update(
    mut commands: Commands,
    mut device: ResMut<RenderDevice>,
    mut queue: ResMut<RenderQueue>,
    pipeline: Res<TerrainComputePipeline>,
    terrain_data: ResMut<RenderAssets<TerrainData>>,
    terrain_query: Query<(Entity, &QuadtreeUpdate, &Handle<TerrainData>)>,
) {
    let terrain_data = terrain_data.into_inner();

    for (entity, update, handle) in terrain_query.iter() {
        let gpu_terrain_data = match terrain_data.get_mut(handle) {
            None => continue,
            Some(gpu_terrain_data) => gpu_terrain_data,
        };
        let quadtree_data = &mut gpu_terrain_data.quadtree_data;

        // insert the node update into the buffer corresponding to its lod
        update.0.iter().for_each(|&data| {
            let lod = TerrainConfig::node_position(data.node_id).0 as usize;
            quadtree_data[lod].0.push(data);
        });

        // create the bind groups for each lod
        let data = quadtree_data
            .iter_mut()
            .map(|(buffer, view)| {
                buffer.write_buffer(&mut device, &mut queue);

                let count = buffer.len() as u32;

                let bind_group = device.create_bind_group(&BindGroupDescriptor {
                    label: None,
                    layout: &pipeline.update_quadtree_bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(view),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: buffer.buffer().unwrap().as_entire_binding(),
                        },
                    ],
                });

                buffer.clear(); // reset buffer for next frame

                (count, bind_group)
            })
            .collect();

        commands.entity(entity).insert(GpuQuadtreeUpdate(data));
    }
}