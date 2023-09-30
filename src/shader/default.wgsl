// vertex shader

struct Uniforms {
    MVP: mat4x4<f32>
}
@binding(0) @group(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec4<f32>
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) vColor: vec4<f32>,
    @location(1) vNormal: vec3<f32>
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = uniforms.MVP * in.position;
    output.vColor = in.color;
    output.vNormal = in.normal.xyz;
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal01: vec3<f32> = in.vNormal * 0.5 + 0.5;
    let color: vec3<f32> = normal01;

    return vec4(color, 1.0);
}