struct NodePosition {
    lod: u32;
    x: u32;
    y: u32;
};

struct NodeList {
    data: array<u32>;
};

struct Patch {
    position: vec2<u32>;
    size: u32;
    atlas_index: u32;
    coord_offset: u32;
    lod: u32;
};

struct PatchList {
    data: array<Patch>;
};

[[group(0), binding(0)]]
var quadtree: texture_2d<u32>;
[[group(0), binding(1)]]
var<storage, read_write> node_list: NodeList;
[[group(0), binding(2)]]
var<storage, read_write> patch_list: PatchList;

fn node_position(id: u32) -> NodePosition {
    return NodePosition((id >> 28u) & 0xFu, (id >> 14u) & 0x3FFFu, id & 0x3FFFu);
}

[[stage(compute), workgroup_size(8, 8, 1)]]
fn build_patch_list(
    [[builtin(workgroup_id)]] workgroup_id: vec3<u32>,
    [[builtin(local_invocation_id)]] invocation_id: vec3<u32>
) {
    let node_index = workgroup_id.x;
    let node_id = node_list.data[node_index];
    let node_position = node_position(node_id);

    let patch_id = invocation_id.xy;
    let patch_size = 4u * (1u << node_position.lod);
    let patch_position = (vec2<u32>(node_position.x, node_position.y) * 8u + patch_id) * patch_size;
    let coord_offset = patch_id.y * 8u + patch_id.x;
    let patch_index = node_index * 64u + coord_offset;
    let atlas_index = textureLoad(quadtree, vec2<i32>(i32(node_position.x), i32(node_position.y)), i32(node_position.lod)).x;

    var patch: Patch;
    patch.position = patch_position;
    patch.size = patch_size;
    patch.atlas_index = atlas_index;
    patch.coord_offset = coord_offset;
    patch.lod = node_position.lod;
    patch_list.data[patch_index] = patch;
}
