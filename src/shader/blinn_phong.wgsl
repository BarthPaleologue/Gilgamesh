// Required Uniforms
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

// Additional Uniforms
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
@group(1) @binding(1) var<uniform> point_lights : array<PointLightUniforms, 64>;
@group(1) @binding(2) var<uniform> point_lights_count : u32;

struct PhongUniforms {
    diffuse_color: vec3<f32>,
    has_diffuse_texture: u32,
    ambient_color: vec3<f32>,
    has_ambient_texture: u32,
    specular_color: vec3<f32>,
    has_specular_texture: u32,
    has_normal_map: u32
}
@group(1) @binding(3) var<uniform> phong: PhongUniforms;

// Textures
@group(2) @binding(0) var diffuse_texture: texture_2d<f32>;
@group(2) @binding(1) var diffuse_sampler: sampler;

@group(2) @binding(2) var ambient_texture: texture_2d<f32>;
@group(2) @binding(3) var ambient_sampler: sampler;

@group(2) @binding(4) var specular_texture: texture_2d<f32>;
@group(2) @binding(5) var specular_sampler: sampler;

@group(2) @binding(6) var normal_texture: texture_2d<f32>;
@group(2) @binding(7) var normal_sampler: sampler;

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var normal = in.vNormal;
    if(phong.has_normal_map > 0u) {
        // do complicated stuff in tangent space
        let normal_map: vec3<f32> = textureSample(normal_texture, normal_sampler, in.vUV).rgb;
        let normal_map_tangent: vec3<f32> = normalize(normal_map * 2.0 - 1.0);
        let normal_map_bitangent: vec3<f32> = normalize(cross(in.vNormal, in.vTangent));

        let normal_map_normal: mat3x3<f32> = mat3x3<f32>(in.vTangent, normal_map_bitangent, normal);
        normal = normalize(normal_map_normal * normal_map_tangent);
    }
    // to world space
    normal = normalize((transform.normal_matrix * vec4<f32>(normal, 0.0)).xyz);

    var diffuse: vec3<f32> = phong.diffuse_color;
    if(phong.has_diffuse_texture > 0u) {
        diffuse = textureSample(diffuse_texture, diffuse_sampler, in.vUV).rgb;
    }

    var ambient: vec3<f32> = phong.ambient_color;
    if(phong.has_ambient_texture > 0u) {
        ambient = textureSample(ambient_texture, ambient_sampler, in.vUV).rgb;
    }

    let view_dir: vec3<f32> = normalize(camera.position - in.vPositionW);

    let reflect_dir: vec3<f32> = reflect(directionalLight.direction, normal);
    let specular_strength: f32 = pow(max(0.0, dot(view_dir, reflect_dir)), 32.0);
    var specular: vec3<f32> = specular_strength * directionalLight.color * directionalLight.intensity * phong.specular_color;
    if(phong.has_specular_texture > 0u) {
        specular = specular * textureSample(specular_texture, specular_sampler, in.vUV).r;
    }

    let ndl = max(0.0, dot(normal, -directionalLight.direction));
    var color = diffuse * directionalLight.color * directionalLight.intensity + specular;
    color *= ndl;

    for (var i: u32 = 0u; i < point_lights_count; i = i + 1u) {
        let light_dir: vec3<f32> = normalize(point_lights[i].position - in.vPositionW);
        let ndl = max(0.0, dot(normal, light_dir));

        let reflect_dir: vec3<f32> = reflect(-light_dir, normal);
        let specular_strength: f32 = pow(max(0.0, dot(view_dir, reflect_dir)), 32.0);
        var specular: vec3<f32> = specular_strength * point_lights[i].color * phong.specular_color;
        if(phong.has_specular_texture > 0u) {
             specular = specular * textureSample(specular_texture, specular_sampler, in.vUV).r;
        }

        color += (diffuse * point_lights[i].color * point_lights[i].intensity + specular) * ndl;
    }

    color += ambient;

    //color = vec3(in.vUV, 1.0);//in.vNormal * 0.5 + 0.5;

    return vec4(color, 1.0);
}