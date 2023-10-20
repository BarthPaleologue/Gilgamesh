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

struct PbrUniforms {
    albedo_color: vec3<f32>,
    has_albedo_texture: u32,
    ambient_color: vec3<f32>,
    has_ambient_texture: u32,
    metallic: f32,
    has_metallic_texture: u32,
    roughness: f32,
    has_roughness_texture: u32,
    has_normal_map: u32
}
@group(1) @binding(3) var<uniform> pbr: PbrUniforms;

// Textures
@group(2) @binding(0) var albedo_texture: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;

@group(2) @binding(2) var ambient_texture: texture_2d<f32>;
@group(2) @binding(3) var ambient_sampler: sampler;

@group(2) @binding(4) var normal_texture: texture_2d<f32>;
@group(2) @binding(5) var normal_sampler: sampler;

@group(2) @binding(6) var metallic_texture: texture_2d<f32>;
@group(2) @binding(7) var metallic_sampler: sampler;

@group(2) @binding(8) var roughness_texture: texture_2d<f32>;
@group(2) @binding(9) var roughness_sampler: sampler;

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

const PI: f32 = 3.14159265359;

fn DistributionGGX(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a      = roughness*roughness;
    let a2     = a*a;
    let NdotH  = max(dot(N, H), 0.0);
    let NdotH2 = NdotH*NdotH;

    let num   = a2;
    var denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / max(denom, 0.001);
}

fn GeometrySchlickGGX(NdotV: f32, roughness: f32) -> f32 {
    let r: f32 = (roughness + 1.0);
    let k: f32 = (r*r) / 8.0;

    let num: f32   = NdotV;
    let denom: f32 = NdotV * (1.0 - k) + k;

    return num / max(denom, 0.001);
}
fn GeometrySmith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2  = GeometrySchlickGGX(NdotV, roughness);
    let ggx1  = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

fn fresnelSchlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

fn pbr_radiance(L: vec3<f32>, H: vec3<f32>, V: vec3<f32>, N: vec3<f32>, light_color: vec3<f32>, distance: f32, albedo_color: vec3<f32>, metallic: f32, roughness: f32) -> vec3<f32> {
    let attenuation: f32 = 1.0 / (distance * distance);
    let radiance: vec3<f32> = light_color * attenuation;

    var F0: vec3<f32> = vec3(0.04);
    F0      = mix(F0, albedo_color, metallic);
    let F: vec3<f32>  = fresnelSchlick(max(dot(H, V), 0.0), F0);

    let NDF: f32 = DistributionGGX(N, H, roughness);
    let G: f32   = GeometrySmith(N, V, L, roughness);

    let numerator: vec3<f32>    = NDF * G * F;
    let denominator: f32 = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0)  + 0.001;
    let specular: vec3<f32>     = numerator / denominator;

    let kS: vec3<f32> = F;
    var kD: vec3<f32> = 1.0 - kS;

    kD *= 1.0 - metallic;

    let NdotL: f32 = max(dot(N, L), 0.0);
    return (kD * albedo_color / PI + specular) * radiance * NdotL;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var normal = in.vNormal;
    if(pbr.has_normal_map > 0u) {
        // do complicated stuff in tangent space
        let normal_map: vec3<f32> = textureSample(normal_texture, normal_sampler, in.vUV).rgb;
        let normal_map_tangent: vec3<f32> = normalize(normal_map * 2.0 - 1.0);
        let normal_map_bitangent: vec3<f32> = normalize(cross(in.vNormal, in.vTangent));

        let normal_map_normal: mat3x3<f32> = mat3x3<f32>(in.vTangent, normal_map_bitangent, normal);
        normal = normalize(normal_map_normal * normal_map_tangent);
    }
    // to world space
    normal = normalize((transform.normal_matrix * vec4<f32>(normal, 0.0)).xyz);

    var albedo: vec3<f32> = pbr.albedo_color;
    if(pbr.has_albedo_texture > 0u) {
        albedo = pow(textureSample(albedo_texture, albedo_sampler, in.vUV).rgb, vec3(2.2));
    }

    var ambient: vec3<f32> = pbr.ambient_color;
    if(pbr.has_ambient_texture > 0u) {
        ambient = textureSample(ambient_texture, ambient_sampler, in.vUV).rgb;
    }

    var metallic: f32 = pbr.metallic;
    var roughness: f32 = pbr.roughness;

    var Lo: vec3<f32> = vec3(0.0);

    let V: vec3<f32> = normalize(camera.position - in.vPositionW);
    let N: vec3<f32> = normal;

    {
        let L: vec3<f32> = -normalize(directionalLight.direction);
        let H: vec3<f32> = normalize(V + L);

        Lo += pbr_radiance(L, H, V, N, directionalLight.color, 1.0, albedo, metallic, roughness) * directionalLight.intensity;
    }

    for (var i: u32 = 0u; i < point_lights_count; i = i + 1u) {
        let L: vec3<f32> = normalize(point_lights[i].position - in.vPositionW);
        let H: vec3<f32> = normalize(V + L);

        let distance: f32 = 1.0; //length(point_lights[i].position - in.vPositionW);

        Lo += pbr_radiance(L, H, V, N, point_lights[i].color, distance, albedo, metallic, roughness) * point_lights[i].intensity;
    }

    var color = Lo + ambient;
    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0/2.2));

    return vec4(color, 1.0);
}