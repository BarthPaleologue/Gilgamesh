@group(0) @binding(0) var source: texture_2d<f32>;

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
