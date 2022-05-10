use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy_terrain::attachment_loader::{TextureAttachmentFromDisk, TextureAttachmentFromDiskLoader};
use bevy_terrain::config::TerrainConfig;
use bevy_terrain::render::gpu_node_atlas::AtlasAttachmentConfig;
use bevy_terrain::AttachmentLabel;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct HeightAttachment;

impl AttachmentLabel for HeightAttachment {
    fn dyn_clone(&self) -> Box<dyn AttachmentLabel> {
        Box::new(HeightAttachment)
    }
}

pub(crate) fn setup_terrain(
    mut config: &mut TerrainConfig,
    mut from_disk_loader: &mut TextureAttachmentFromDiskLoader,
) {
    setup_sampler(config, "sampler".into());
    setup_height(config, from_disk_loader, "height".into(), 129);
    setup_albedo(config, from_disk_loader, "albedo".into(), 128 * 5);
}

fn setup_sampler(config: &mut TerrainConfig, label: String) {
    let sampler_descriptor = SamplerDescriptor {
        label: "sampler_attachment".into(),
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        ..default()
    };

    config.add_attachment(
        label,
        AtlasAttachmentConfig::Sampler {
            binding: 2,
            sampler_descriptor,
        },
    );
}

fn setup_height(
    mut config: &mut TerrainConfig,
    mut from_disk_loader: &mut TextureAttachmentFromDiskLoader,
    label: String,
    texture_size: u32,
) {
    let atlas_texture_descriptor = TextureDescriptor {
        label: "atlas_height_attachment_texture".into(),
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
        label: "height_attachment_view".into(),
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
    };

    let mut node_texture_descriptor = atlas_texture_descriptor.clone();
    node_texture_descriptor.label = "node_height_attachment_texture".into();
    node_texture_descriptor.size.depth_or_array_layers = 1;
    node_texture_descriptor.usage |= TextureUsages::COPY_SRC;

    from_disk_loader.add_attachment(
        label.clone(),
        TextureAttachmentFromDisk {
            path: "output/height".into(),
            texture_descriptor: node_texture_descriptor,
        },
    );

    config.add_attachment(
        label.clone(),
        AtlasAttachmentConfig::Texture {
            binding: 3,
            texture_size,
            texture_descriptor: atlas_texture_descriptor,
            view_descriptor,
        },
    );
}

fn setup_albedo(
    mut config: &mut TerrainConfig,
    mut from_disk_loader: &mut TextureAttachmentFromDiskLoader,
    label: String,
    texture_size: u32,
) {
    let atlas_texture_descriptor = TextureDescriptor {
        label: "albedo_attachment_texture".into(),
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
        label: "albedo_attachment_view".into(),
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
    };

    let mut node_texture_descriptor = atlas_texture_descriptor.clone();
    node_texture_descriptor.label = "node_albedo_attachment_texture".into();
    node_texture_descriptor.size.depth_or_array_layers = 1;
    node_texture_descriptor.usage |= TextureUsages::COPY_SRC;

    from_disk_loader.add_attachment(
        label.clone(),
        TextureAttachmentFromDisk {
            path: "output/albedo".into(),
            texture_descriptor: node_texture_descriptor,
        },
    );

    config.add_attachment(
        label.clone(),
        AtlasAttachmentConfig::Texture {
            binding: 4,
            texture_size,
            texture_descriptor: atlas_texture_descriptor,
            view_descriptor,
        },
    );
}
