use naga::valid::{Capabilities, ValidationFlags, Validator};

fn validate_shader(source: &str) -> naga::Module {
    let module = naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(source)));
    Validator::new(ValidationFlags::all(), Capabilities::empty())
        .validate(&module)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(source)));
    module
}

#[test]
fn pbr_shader_is_valid() {
    validate_shader(include_str!("../src/shader/pbr.wgsl"));
}

#[test]
fn rendering_shaders_translate_to_webgl2() {
    use naga::back::glsl;

    let fullscreen_sources = [
        include_str!("../src/blocks/tone_map.wgsl"),
        include_str!("../src/blocks/expose.wgsl"),
        include_str!("../src/blocks/resize_nearest.wgsl"),
        include_str!("../src/blocks/resize_bilinear.wgsl"),
        include_str!("../src/blocks/bloom.wgsl"),
        include_str!("../src/blocks/pyramid_blur/merge_levels.wgsl"),
        include_str!("../src/blocks/pyramid_blur/downsample.wgsl"),
        include_str!("../src/blocks/pyramid_blur/upsample.wgsl"),
        include_str!("../src/blocks/visualize_depth.wgsl"),
        include_str!("../src/blocks/vertical_split_screen.wgsl"),
        include_str!("../src/core/image_blit.wgsl"),
    ]
    .map(|fragment| [include_str!("../src/core/fullscreen_pass.wgsl"), fragment].join("\n"));
    for source in fullscreen_sources
        .into_iter()
        .chain([include_str!("../src/shader/pbr.wgsl").to_owned()])
    {
        let module = validate_shader(&source);
        let info = Validator::new(ValidationFlags::all(), Capabilities::empty())
            .validate(&module)
            .unwrap();
        let options = glsl::Options {
            version: glsl::Version::Embedded {
                version: 300,
                is_webgl: true,
            },
            ..Default::default()
        };
        for entry in &module.entry_points {
            glsl::Writer::new(
                String::new(),
                &module,
                &info,
                &options,
                &glsl::PipelineOptions {
                    shader_stage: entry.stage,
                    entry_point: entry.name.clone(),
                    multiview: None,
                },
                naga::proc::BoundsCheckPolicies::default(),
            )
            .unwrap()
            .write()
            .unwrap_or_else(|error| panic!("{}: {error}", entry.name));
        }
    }
}

#[test]
fn uniform_bindings_have_webgl_compatible_sizes() {
    let source = include_str!("../src/shader/pbr.wgsl");
    let module = validate_shader(source);
    let mut layouter = naga::proc::Layouter::default();
    layouter.update(module.to_ctx()).unwrap();
    for (_, variable) in module.global_variables.iter() {
        if variable.space == naga::AddressSpace::Uniform {
            assert_eq!(layouter[variable.ty].size % 16, 0, "{:?}", variable.name);
            if variable.name.as_deref() == Some("transform") {
                assert_eq!(
                    layouter[variable.ty].size as usize,
                    std::mem::size_of::<gilgamesh::transform::TransformUniforms>()
                );
            }
            let expected_size = match variable.name.as_deref() {
                Some("directional_lights") => Some(
                    gilgamesh::settings::MAX_DIRECTIONAL_LIGHTS
                        * std::mem::size_of::<
                            gilgamesh::lights::directional_light::DirectionalLightUniform,
                        >(),
                ),
                Some("point_lights") => Some(
                    gilgamesh::settings::MAX_POINT_LIGHTS
                        * std::mem::size_of::<gilgamesh::lights::point_light::PointLightUniforms>(),
                ),
                Some("camera") => Some(std::mem::size_of::<
                    gilgamesh::camera::uniforms::CameraUniforms,
                >()),
                Some("light_counts") => Some(std::mem::size_of::<[u32; 4]>()),
                _ => None,
            };
            if let Some(expected_size) = expected_size {
                assert_eq!(layouter[variable.ty].size as usize, expected_size);
            }
        }
    }
}

#[test]
fn shader_bindings_separate_frame_material_and_object_data() {
    let source = include_str!("../src/shader/pbr.wgsl");
    let module = validate_shader(source);
    let bindings: std::collections::BTreeSet<_> = module
        .global_variables
        .iter()
        .filter_map(|(_, variable)| variable.binding.as_ref())
        .map(|binding| (binding.group, binding.binding))
        .collect();
    let expected: std::collections::BTreeSet<_> = (0..4)
        .map(|binding| (0, binding))
        .chain((0..=10).map(|binding| (1, binding)))
        .chain([(2, 0)])
        .collect();
    assert_eq!(bindings, expected);
    for (name, group, binding) in [
        ("camera", 0, 0),
        ("directional_lights", 0, 1),
        ("point_lights", 0, 2),
        ("light_counts", 0, 3),
        ("transform", 2, 0),
    ] {
        let variable = module
            .global_variables
            .iter()
            .find(|(_, variable)| variable.name.as_deref() == Some(name))
            .unwrap()
            .1;
        let resource = variable.binding.as_ref().unwrap();
        assert_eq!(
            (resource.group, resource.binding),
            (group, binding),
            "{name}"
        );
    }
}
