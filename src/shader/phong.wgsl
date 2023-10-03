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
@group(1) @binding(0)
var<uniform> directionalLight: DirectionalLightUniforms;

struct PointLightUniforms {
    color: vec3<f32>,
    position: vec3<f32>,
    intensity: f32
}
@group(1) @binding(1) var<uniform> point_lights : array<PointLightUniforms, 8>;
@group(1) @binding(2) var<uniform> point_lights_count : u32;

struct PhongUniforms {
    diffuse_color: vec3<f32>,
    has_diffuse_texture: u32,
    ambient_color: vec3<f32>,
    has_ambient_texture: u32,
    specular_color: vec3<f32>,
    has_specular_texture: u32,
}
@group(1) @binding(3) var<uniform> phong: PhongUniforms;

@group(2) @binding(0) var t_diffuse: texture_2d<f32>;
@group(2) @binding(1) var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) uv: vec2<f32>
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) vPositionW: vec3<f32>,
    @location(1) vColor: vec3<f32>,
    @location(2) vNormal: vec3<f32>,
    @location(3) vNormalW: vec3<f32>,
    @location(4) vUV: vec2<f32>
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera.proj_view * transform.world_matrix * vec4<f32>(in.position, 1.0);
    output.vPositionW = (transform.world_matrix * vec4<f32>(in.position, 1.0)).xyz;
    output.vColor = in.color;
    output.vNormal = in.normal;
    output.vNormalW = (transform.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz;
    output.vUV = in.uv;
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var diffuse: vec3<f32> = phong.diffuse_color;
    if(phong.has_diffuse_texture > 0u) {
        diffuse = textureSample(t_diffuse, s_diffuse, in.vUV).rgb;
    }

    let ambient: vec3<f32> = phong.ambient_color;


    let view_dir: vec3<f32> = normalize(camera.position - in.vPositionW);

    let reflect_dir: vec3<f32> = reflect(directionalLight.direction, in.vNormalW);
    let specular_strength: f32 = pow(max(0.0, dot(view_dir, reflect_dir)), 32.0);
    let specular: vec3<f32> = specular_strength * directionalLight.color * directionalLight.intensity * phong.specular_color;

    let ndl = max(0.0, dot(in.vNormalW, -directionalLight.direction));
    var color = diffuse * ndl * directionalLight.color * directionalLight.intensity + specular;

    for (var i: u32 = 0u; i < point_lights_count; i = i + 1u) {
        let light_dir: vec3<f32> = normalize(point_lights[i].position - in.vPositionW);
        let ndl = max(0.0, dot(in.vNormalW, light_dir));

        let reflect_dir: vec3<f32> = reflect(-light_dir, in.vNormalW);
        let specular_strength: f32 = pow(max(0.0, dot(view_dir, reflect_dir)), 32.0);
        let specular: vec3<f32> = specular_strength * point_lights[i].color * phong.specular_color;

        color = color + diffuse * ndl * point_lights[i].color * point_lights[i].intensity + specular;
    }

    color = color + ambient;

    return vec4(color, 1.0);
}