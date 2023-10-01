// vertex shader

struct TransformUniforms {
    position: vec3<f32>,
    world_matrix: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> transform: TransformUniforms;

struct CameraUniforms {
    proj_view: mat4x4<f32>,
    position: vec3<f32>
}
@group(0) @binding(1)
var<uniform> camera: CameraUniforms;

struct DirectionalLightUniforms {
    color: vec3<f32>,
    direction: vec3<f32>,
    intensity: f32
}
@group(0) @binding(2)
var<uniform> directionalLight: DirectionalLightUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) uv: vec2<f32>
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) vColor: vec3<f32>,
    @location(1) vNormal: vec3<f32>,
    @location(2) vNormalW: vec3<f32>,
    @location(3) vUV: vec2<f32>
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera.proj_view * transform.world_matrix * vec4<f32>(in.position, 1.0);
    output.vColor = in.color;
    output.vNormal = in.normal;
    output.vNormalW = (transform.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz;
    output.vUV = in.uv;
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal01: vec3<f32> = in.vNormal * 0.5 + 0.5;

    let diffuse: vec3<f32> = normal01;

    let ndl = max(0.0, dot(in.vNormalW, -directionalLight.direction));
    let color = diffuse * ndl * directionalLight.color;

    return vec4(color, 1.0);
}