@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> destination_size: vec4u;

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
    let uv = position.xy / vec2f(destination_size.xy);
    let texel = 1.0 / vec2f(textureDimensions(source));
    var color = textureSampleLevel(source, source_sampler, uv, 0.0) * 0.125;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            if x == 0 && y == 0 { continue; }
            let weight = select(0.03125, 0.0625, x == 0 || y == 0);
            color += textureSampleLevel(source, source_sampler, uv + vec2f(f32(x), f32(y)) * texel * 2.0, 0.0) * weight;
        }
    }
    for (var y = -1; y <= 1; y += 2) {
        for (var x = -1; x <= 1; x += 2) {
            color += textureSampleLevel(source, source_sampler, uv + vec2f(f32(x), f32(y)) * texel, 0.0) * 0.125;
        }
    }
    return color;
}
