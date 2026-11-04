@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> destination_size: vec4u;

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
    let uv = position.xy / vec2f(destination_size.xy);
    return textureSampleLevel(source, source_sampler, uv, 0.0);
}
