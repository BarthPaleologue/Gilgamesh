// vertex shader

struct Uniforms {
    MVP: mat4x4<f32>
}
@binding(0) @group(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) pos: vec4<f32>,
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
    output.position = uniforms.MVP * in.pos;
    output.vColor = in.color;
    output.vNormal = in.normal.xyz;
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir: vec3<f32> = normalize(vec3<f32>(0.5, 1.0, 1.0));
    let normal01: vec3<f32> = in.vNormal;

    let ndl: f32 = max(dot(in.vNormal, light_dir), 0.02);

    let color: vec3<f32> = ndl * normal01; //in.vColor.xyz * ndl;

    return vec4(color, 1.0);
}