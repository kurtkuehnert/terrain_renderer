struct NodePosition {
    lod: u32;
    x: u32;
    y: u32;
};

struct NodeUpdate {
    node_id: u32;
    atlas_index: u32;
};

struct QuadtreeUpdates {
    data: array<NodeUpdate>;
};

[[group(0), binding(0)]]
var quadtree: texture_storage_2d<r16uint, write>;
[[group(0), binding(1)]]
var<storage> quadtree_updates: QuadtreeUpdates;

fn node_position(id: u32) -> NodePosition {
    return NodePosition((id >> 28u) & 0xFu, (id >> 14u) & 0x3FFFu, id & 0x3FFFu);
}

[[stage(compute), workgroup_size(8, 8, 1)]]
fn update_quadtree(
    [[builtin(global_invocation_id)]] invocation_id: vec3<u32>
) {
    let index = invocation_id.x;
    let update = quadtree_updates.data[index];
    let position = node_position(update.node_id);

    textureStore(quadtree, vec2<i32>(i32(position.x), i32(position.y)), vec4<u32>(update.atlas_index));
}
