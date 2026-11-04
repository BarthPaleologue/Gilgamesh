@group(0) @binding(0) var previous: texture_2d<f32>;
@group(0) @binding(1) var current: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = vec2<i32>(textureDimensions(previous));
    let p = vec2<i32>(id.xy);
    if any(p >= size) {
        return;
    }
    var neighbors = 0u;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            if x == 0 && y == 0 {
                continue;
            }
            let neighbor = (p + vec2(x, y) + size) % size;
            neighbors += u32(textureLoad(previous, neighbor, 0).r > 0.5);
        }
    }
    let alive = textureLoad(previous, p, 0).r > 0.5;
    let survives = neighbors == 3u || (alive && neighbors == 2u);
    textureStore(current, p, vec4(vec3(f32(survives)), 1.0));
}
