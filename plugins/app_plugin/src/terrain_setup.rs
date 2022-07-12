#![allow(dead_code)]
#![allow(unused_variables)]

use bevy::{prelude::*, render::render_resource::*};
use bevy_terrain::{
    attachment::{AtlasAttachmentConfig, AttachmentIndex},
    attachment_loader::{TextureAttachmentFromDisk, TextureAttachmentFromDiskLoader},
    terrain::TerrainConfig,
};

pub(crate) fn setup_default_sampler(config: &mut TerrainConfig, attachment_index: AttachmentIndex) {
    let sampler_descriptor = SamplerDescriptor {
        label: "default_sampler_attachment".into(),
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        ..default()
    };

    config.add_attachment(
        attachment_index,
        AtlasAttachmentConfig::Sampler { sampler_descriptor },
    );
}

pub(crate) fn setup_height_texture(
    config: &mut TerrainConfig,
    from_disk_loader: &mut TextureAttachmentFromDiskLoader,
    attachment_index: AttachmentIndex,
    texture_size: u32,
) {
    let atlas_texture_descriptor = TextureDescriptor {
        label: "atlas_height_texture_attachment".into(),
        size: Extent3d {
            width: texture_size,
            height: texture_size,
            depth_or_array_layers: config.node_atlas_size as u32,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::R16Unorm,
        usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
    };

    let view_descriptor = TextureViewDescriptor {
        label: "height_texture_attachment_view".into(),
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
    };

    let mut node_texture_descriptor = atlas_texture_descriptor.clone();
    node_texture_descriptor.label = "node_height_texture_attachment".into();
    node_texture_descriptor.size.depth_or_array_layers = 1;
    node_texture_descriptor.usage |= TextureUsages::COPY_SRC;

    from_disk_loader.add_attachment(
        attachment_index,
        TextureAttachmentFromDisk {
            path: config.path.clone() + "data/height",
            texture_descriptor: node_texture_descriptor,
        },
    );

    config.add_attachment(
        attachment_index,
        AtlasAttachmentConfig::Texture {
            texture_size,
            texture_descriptor: atlas_texture_descriptor,
            view_descriptor,
        },
    );
}

pub(crate) fn setup_albedo_texture(
    config: &mut TerrainConfig,
    from_disk_loader: &mut TextureAttachmentFromDiskLoader,
    attachment_index: AttachmentIndex,
    texture_size: u32,
) {
    let atlas_texture_descriptor = TextureDescriptor {
        label: "atlas_albedo_texture_attachment".into(),
        size: Extent3d {
            width: texture_size,
            height: texture_size,
            depth_or_array_layers: config.node_atlas_size as u32,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
    };

    let view_descriptor = TextureViewDescriptor {
        label: "albedo_texture_attachment_view".into(),
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
    };

    let mut node_texture_descriptor = atlas_texture_descriptor.clone();
    node_texture_descriptor.label = "node_albedo_texture_attachment".into();
    node_texture_descriptor.size.depth_or_array_layers = 1;
    node_texture_descriptor.usage |= TextureUsages::COPY_SRC;

    from_disk_loader.add_attachment(
        attachment_index,
        TextureAttachmentFromDisk {
            path: config.path.clone() + "data/albedo",
            texture_descriptor: node_texture_descriptor,
        },
    );

    config.add_attachment(
        attachment_index,
        AtlasAttachmentConfig::Texture {
            texture_size,
            texture_descriptor: atlas_texture_descriptor,
            view_descriptor,
        },
    );
}

pub(crate) fn setup_density_texture(
    config: &mut TerrainConfig,
    from_disk_loader: &mut TextureAttachmentFromDiskLoader,
    attachment_index: AttachmentIndex,
    texture_size: u32,
) {
    let atlas_texture_descriptor = TextureDescriptor {
        label: "atlas_density_texture_attachment".into(),
        size: Extent3d {
            width: texture_size,
            height: texture_size,
            depth_or_array_layers: config.node_atlas_size as u32,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::R16Unorm,
        usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
    };

    let view_descriptor = TextureViewDescriptor {
        label: "height_density_attachment_view".into(),
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
    };

    let mut node_texture_descriptor = atlas_texture_descriptor.clone();
    node_texture_descriptor.label = "node_density_texture_attachment".into();
    node_texture_descriptor.size.depth_or_array_layers = 1;
    node_texture_descriptor.usage |= TextureUsages::COPY_SRC;

    from_disk_loader.add_attachment(
        attachment_index,
        TextureAttachmentFromDisk {
            path: config.path.clone() + "data/density",
            texture_descriptor: node_texture_descriptor,
        },
    );

    config.add_attachment(
        attachment_index,
        AtlasAttachmentConfig::Texture {
            texture_size,
            texture_descriptor: atlas_texture_descriptor,
            view_descriptor,
        },
    );
}
