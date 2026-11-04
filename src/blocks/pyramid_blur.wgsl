@group(0) @binding(0) var fine: texture_2d<f32>;
@group(0) @binding(1) var coarse: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
    let p = vec2i(position.xy);
    return (textureLoad(fine, p, 0) + textureLoad(coarse, p, 0)) * 0.5;
}
