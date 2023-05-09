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
    @location(0) vPosition: vec3<f32>,
    @location(1) vColor: vec4<f32>,
    @location(2) vNormal: vec3<f32>
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = uniforms.MVP * in.pos;
    output.vPosition = in.pos.xyz;
    output.vColor = in.color;
    output.vNormal = in.normal.xyz;
    return output;
}

struct FragUniforms {
    light_dir: vec3<f32>,
    camera_position: vec3<f32>,
    max_height: f32
};
@binding(1) @group(0) var<uniform> frag_uniforms : FragUniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal01: vec3<f32> = normalize(in.vNormal);

    let height01 = in.vPosition.y / frag_uniforms.max_height;
    let grass_color = vec3(0.0, 0.5, 0.0);
    let snow_color = vec3(1.0, 1.0, 1.0);

    let flat_color = mix(grass_color, snow_color, smoothstep(0.7, 0.8, height01));

    let slope = 1.0 - pow(dot(normalize(in.vNormal), vec3(0.0, 1.0, 0.0)), 32.0);
    let slope_color = vec3(0.2, 0.1, 0.1);

    let ndl: f32 = max(dot(in.vNormal, normalize(frag_uniforms.light_dir)), 0.01);

    let color: vec3<f32> = mix(flat_color, slope_color, smoothstep(0.8, 0.9, slope));

    return vec4(ndl * color, 1.0);
}