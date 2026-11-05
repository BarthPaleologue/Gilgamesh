@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var<uniform> exposure: vec4<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let color = textureLoad(source, vec2<i32>(position.xy), 0);
    return vec4(color.rgb * exposure.x, color.a);
}
