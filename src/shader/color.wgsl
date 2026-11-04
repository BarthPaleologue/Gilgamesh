@group(0) @binding(0) var source: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(index & 1u) * 4.0 - 1.0;
    let y = f32((index >> 1u) & 1u) * 4.0 - 1.0;
    return vec4(x, y, 0.0, 1.0);
}

@fragment
fn tone_mapping(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let color = textureLoad(source, vec2<i32>(position.xy), 0);
    let rgb = max(color.rgb, vec3(0.0));
    return vec4(rgb / (rgb + vec3(1.0)), color.a);
}

@fragment
fn image_blit(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return textureLoad(source, vec2<i32>(position.xy), 0);
}

@fragment
fn encode_srgb(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let color = textureLoad(source, vec2<i32>(position.xy), 0);
    let encoded = select(
        12.92 * color.rgb,
        1.055 * pow(color.rgb, vec3(1.0 / 2.4)) - vec3(0.055),
        color.rgb > vec3(0.0031308),
    );
    return vec4(encoded, color.a);
}
