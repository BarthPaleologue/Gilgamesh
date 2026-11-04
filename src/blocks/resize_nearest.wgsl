@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var<uniform> destination_size: vec4<u32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let size = textureDimensions(source);
    let uv = position.xy / vec2<f32>(destination_size.xy);
    let p = min(vec2<u32>(uv * vec2<f32>(size)), size - vec2(1u));
    return textureLoad(source, vec2<i32>(p), 0);
}
