@group(0) @binding(0) var depth: texture_depth_2d;
@group(0) @binding(1) var depth_sampler: sampler;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / vec2<f32>(textureDimensions(depth));
    let value = textureSampleLevel(depth, depth_sampler, uv, 0);
    return vec4(vec3(value), 1.0);
}
