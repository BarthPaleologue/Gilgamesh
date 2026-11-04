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
fn blinn_phong_shader_is_valid() {
    validate_shader(include_str!("../src/shader/blinn_phong.wgsl"));
}

#[test]
fn pbr_shader_is_valid() {
    validate_shader(include_str!("../src/shader/pbr.wgsl"));
}

#[test]
fn uniform_bindings_have_webgl_compatible_sizes() {
    for source in [
        include_str!("../src/shader/blinn_phong.wgsl"),
        include_str!("../src/shader/pbr.wgsl"),
    ] {
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
            }
        }
    }
}

#[test]
fn shader_bindings_separate_frame_material_and_object_data() {
    for (source, texture_count) in [
        (include_str!("../src/shader/blinn_phong.wgsl"), 4),
        (include_str!("../src/shader/pbr.wgsl"), 5),
    ] {
        let module = validate_shader(source);
        let bindings: std::collections::BTreeSet<_> = module
            .global_variables
            .iter()
            .filter_map(|(_, variable)| variable.binding.as_ref())
            .map(|binding| (binding.group, binding.binding))
            .collect();
        let expected: std::collections::BTreeSet<_> = (0..4)
            .map(|binding| (0, binding))
            .chain((0..1 + 2 * texture_count).map(|binding| (1, binding)))
            .chain([(2, 0)])
            .collect();
        assert_eq!(bindings, expected);
        for (name, group, binding) in [
            ("camera", 0, 0),
            ("directionalLight", 0, 1),
            ("point_lights", 0, 2),
            ("point_lights_count", 0, 3),
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
}
