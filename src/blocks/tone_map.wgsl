@group(0) @binding(0) var source: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let color = textureLoad(source, vec2<i32>(position.xy), 0);
    return vec4(khronos_pbr_neutral(max(color.rgb, vec3(0.0))), color.a);
}

// Copyright 2024 The Khronos Group, Inc.
// SPDX-License-Identifier: Apache-2.0
// Adapted from https://github.com/KhronosGroup/ToneMapping/blob/main/PBR_Neutral/pbrNeutral.glsl
fn khronos_pbr_neutral(input: vec3<f32>) -> vec3<f32> {
    const start_compression = 0.8 - 0.04;
    const desaturation = 0.15;

    let minimum = min(input.r, min(input.g, input.b));
    let offset = select(0.04, minimum - 6.25 * minimum * minimum, minimum < 0.08);
    var color = input - vec3(offset);
    let peak = max(color.r, max(color.g, color.b));
    if peak < start_compression {
        return color;
    }

    const d = 1.0 - start_compression;
    let new_peak = 1.0 - d * d / (peak + d - start_compression);
    color *= new_peak / peak;
    let highlight_desaturation = 1.0 - 1.0 / (desaturation * (peak - new_peak) + 1.0);
    return mix(color, vec3(new_peak), highlight_desaturation);
}
