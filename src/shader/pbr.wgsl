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

const PI = 3.14159265359;

fn DistributionGGX(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a      = roughness*roughness;
    let a2     = a*a;
    let NdotH  = max(dot(N, H), 0.0);
    let NdotH2 = NdotH*NdotH;

    let num   = a2;
    var denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

fn GeometrySchlickGGX(NdotV: f32, roughness: f32) -> f32 {
    let r: f32 = (roughness + 1.0);
    let k: f32 = (r*r) / 8.0;

    let num: f32   = NdotV;
    let denom: f32 = NdotV * (1.0 - k) + k;

    return num / denom;
}
fn GeometrySmith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2  = GeometrySchlickGGX(NdotV, roughness);
    let ggx1  = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

fn fresnelSchlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

fn pbr_radiance(L: vec3<f32>, H: vec3<f32>, V: vec3<f32>, N: vec3<f32>, distance: f32) -> vec3<f32> {
    return vec3(0.7, 0.3, 0.3);
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
    if(pbr.has_diffuse_texture > 0u) {
        albedo = textureSample(albedo_texture, albedo_sampler, in.vUV).rgb;
    }

    var ambient: vec3<f32> = pbr.ambient_color;
    if(pbr.has_ambient_texture > 0u) {
        ambient = textureSample(ambient_texture, ambient_sampler, in.vUV).rgb;
    }

    let V: vec3<f32> = normalize(camera.position - in.vPositionW);
    let L: vec3<f32> = normalize(directionalLight.direction);

    let H: vec3<f32> = normalize(V + L);

    let radiance: vec3<f32> = pbr_radiance(L, H, V, normal, 1.0);

    /*let reflect_dir: vec3<f32> = reflect(directionalLight.direction, normal);
    let specular_strength: f32 = pow(max(0.0, dot(V, reflect_dir)), 32.0);
    var specular: vec3<f32> = specular_strength * directionalLight.color * directionalLight.intensity * pbr.specular_color;
    if(pbr.has_specular_texture > 0u) {
        specular = specular * textureSample(specular_texture, specular_sampler, in.vUV).r;
    }

    var ndl = max(0.0, dot(normal, -directionalLight.direction));
    var color = albedo * directionalLight.color * directionalLight.intensity + specular;

    for (var i: u32 = 0u; i < point_lights_count; i = i + 1u) {
        let light_dir: vec3<f32> = normalize(point_lights[i].position - in.vPositionW);
        ndl = ndl + max(0.0, dot(normal, light_dir));

        let reflect_dir: vec3<f32> = reflect(-light_dir, normal);
        let specular_strength: f32 = pow(max(0.0, dot(V, reflect_dir)), 32.0);
        var specular: vec3<f32> = specular_strength * point_lights[i].color * pbr.specular_color;
        if(pbr.has_specular_texture > 0u) {
             specular = specular * textureSample(specular_texture, specular_sampler, in.vUV).r;
        }

        color = color + albedo * point_lights[i].color * point_lights[i].intensity + specular;
    }

    color = color * ndl + ambient;*/

    let color = radiance * albedo + ambient;

    //color = vec3(in.vUV, 1.0);//in.vNormal * 0.5 + 0.5;

    return vec4(color, 1.0);
}