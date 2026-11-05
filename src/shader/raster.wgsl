// Required Uniforms
struct TransformUniforms {
    position: vec3<f32>,
    world_matrix: mat4x4<f32>,
}
@group(2) @binding(0)
var<uniform> transform: TransformUniforms;

struct CameraUniforms {
    proj_view: mat4x4<f32>,
    position: vec3<f32>
}
@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

// Additional Uniforms
struct DirectionalLightUniforms {
    color: vec3<f32>,
    direction: vec3<f32>,
    intensity: f32
}
@group(0) @binding(1)
var<uniform> directional_lights: array<DirectionalLightUniforms, 64>;

struct PointLightUniforms {
    color: vec3<f32>,
    position: vec3<f32>,
    intensity: f32
}
@group(0) @binding(2) var<uniform> point_lights : array<PointLightUniforms, 64>;
struct LightCounts {
    directional: u32,
    point: u32,
    _padding: vec2<u32>,
}
@group(0) @binding(3) var<uniform> light_counts: LightCounts;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) uv: vec2<f32>
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) vPositionW: vec3<f32>,
    @location(1) vColor: vec3<f32>,
    @location(2) vNormal: vec3<f32>,
    @location(3) vTangent: vec3<f32>,
    @location(4) vUV: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera.proj_view * transform.world_matrix * vec4<f32>(in.position, 1.0);
    output.vPositionW = (transform.world_matrix * vec4<f32>(in.position, 1.0)).xyz;
    output.vColor = in.color;
    output.vNormal = in.normal;
    output.vTangent = in.tangent;
    output.vUV = in.uv;
    return output;
}
