@group(0) @binding(0) var state: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> seed: vec4<u32>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(state);
    if any(id.xy >= size) {
        return;
    }
    let uv = vec2<f32>(id.xy) / vec2<f32>(size);
    let noise = fract(sin(dot(uv, vec2(12.9898, 78.233)) + f32(seed.x)) * 43758.5453);
    let alive = f32(noise > 0.4);
    textureStore(state, vec2<i32>(id.xy), vec4(vec3(alive), 1.0));
}
