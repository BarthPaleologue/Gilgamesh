@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var blurred: texture_2d<f32>;
@group(0) @binding(2) var<uniform> weight: vec4f;

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
    let p = vec2i(position.xy);
    let original = textureLoad(source, p, 0);
    let diffuse = textureLoad(blurred, p, 0);
    return vec4f(mix(original.rgb, diffuse.rgb, weight.x), original.a);
}
