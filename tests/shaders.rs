use naga::valid::{Capabilities, ValidationFlags, Validator};

fn validate_shader(source: &str) {
    let module = naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(source)));
    Validator::new(ValidationFlags::all(), Capabilities::empty())
        .validate(&module)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(source)));
}

#[test]
fn blinn_phong_shader_is_valid() {
    validate_shader(include_str!("../src/shader/blinn_phong.wgsl"));
}

#[test]
fn pbr_shader_is_valid() {
    validate_shader(include_str!("../src/shader/pbr.wgsl"));
}
