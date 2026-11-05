@group(0) @binding(0) var foreground: texture_2d<f32>;
@group(0) @binding(1) var background: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / vec2<f32>(textureDimensions(background));
    let fg_position = vec2<i32>(uv * vec2<f32>(textureDimensions(foreground)));
    let fg = textureLoad(foreground, fg_position, 0);
    let bg = textureLoad(background, vec2<i32>(position.xy), 0);
    return fg + bg * (1.0 - fg.a);
}
