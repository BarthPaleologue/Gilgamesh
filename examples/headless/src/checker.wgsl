struct CheckerUniforms {
    a: vec4<f32>,
    b: vec4<f32>,
    scale: vec4<f32>,
}
@group(1) @binding(0) var<uniform> checker: CheckerUniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let cell = floor(in.vUV * checker.scale.xy);
    let odd = (i32(cell.x) + i32(cell.y)) % 2 != 0;
    return select(checker.a, checker.b, odd);
}
