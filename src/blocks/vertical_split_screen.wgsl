@group(0) @binding(0) var left: texture_2d<f32>;
@group(0) @binding(1) var right: texture_2d<f32>;
@group(0) @binding(2) var<uniform> split: vec4<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    if position.x < split.x * f32(textureDimensions(left).x) {
        return textureLoad(left, vec2<i32>(position.xy), 0);
    }
    let uv = position.xy / vec2<f32>(textureDimensions(left));
    return textureLoad(right, vec2<i32>(uv * vec2<f32>(textureDimensions(right))), 0);
}
