use super::*;
use std::cell::Cell;
use std::rc::Rc;
use wgpu::TextureUsages as Usage;

#[test]
fn blocks_compose_in_nested_calls_without_a_gpu() {
    use crate::blocks::{rasterize, tone_map};
    use crate::camera::Camera;
    use crate::core::render_context::FrameBindingsData;
    use crate::lights::directional_light::DirectionalLight;

    let camera: Camera = Camera::default();
    let light = DirectionalLight::default();
    let mut graph = RenderGraph::new();
    let output = tone_map(
        rasterize(
            FrameBindingsData::new(&camera, &light, &[]),
            [],
            TextureFormat::Rgba16Float,
            &mut graph,
        )
        .color,
        TextureFormat::Rgba8UnormSrgb,
        &mut graph,
    );
    graph.export(output);
    graph.validate().unwrap();
}

fn image(label: &str, graph: &mut RenderGraph<'_>) -> Image {
    graph.create_image(
        label,
        ImageDescriptor {
            size: ImageSize::Target,
            format: TextureFormat::Rgba16Float,
        },
    )
}

#[test]
fn dependencies_follow_images_and_union_usages_with_multiple_inputs_and_outputs() {
    let mut graph = RenderGraph::new();
    let a = image("A", &mut graph);
    let b = image("B", &mut graph);
    let result = image("Result", &mut graph);
    graph.add_pass(
        "Merge",
        &[(a, Usage::TEXTURE_BINDING), (b, Usage::TEXTURE_BINDING)],
        &[(result, Usage::RENDER_ATTACHMENT)],
        |_| {},
    );
    graph.add_pass(
        "Produce",
        &[],
        &[(a, Usage::STORAGE_BINDING), (b, Usage::STORAGE_BINDING)],
        |_| {},
    );
    graph.export(result);
    let plan = graph.compile().unwrap();
    assert_eq!(plan.order, [1, 0]);
    assert_eq!(
        plan.usages[a.index],
        Usage::STORAGE_BINDING | Usage::TEXTURE_BINDING
    );
    assert_eq!(
        plan.usages[result.index],
        Usage::RENDER_ATTACHMENT | Usage::TEXTURE_BINDING | Usage::COPY_SRC
    );
}

#[test]
fn rejects_missing_or_multiple_writers() {
    let mut graph = RenderGraph::new();
    let a = image("Uninitialized", &mut graph);
    graph.export(a);
    assert!(
        graph
            .validate()
            .unwrap_err()
            .to_string()
            .contains("has no writer")
    );
    graph.add_pass("First", &[], &[(a, Usage::RENDER_ATTACHMENT)], |_| {});
    graph.add_pass("Second", &[], &[(a, Usage::STORAGE_BINDING)], |_| {});
    assert!(
        graph
            .validate()
            .unwrap_err()
            .to_string()
            .contains("multiple writers")
    );
}

#[test]
fn rejects_foreign_handles_even_when_indices_match() {
    let mut first = RenderGraph::new();
    let foreign = image("Foreign", &mut first);
    let mut graph = RenderGraph::new();
    let output = image("Output", &mut graph);
    graph.add_pass(
        "Bad Read",
        &[(foreign, Usage::TEXTURE_BINDING)],
        &[(output, Usage::RENDER_ATTACHMENT)],
        |_| {},
    );
    graph.export(output);
    assert!(
        graph
            .validate()
            .unwrap_err()
            .to_string()
            .contains("another graph")
    );
}

#[test]
fn rejects_cycles_and_same_pass_feedback() {
    let mut graph = RenderGraph::new();
    let a = image("A", &mut graph);
    let b = image("B", &mut graph);
    graph.add_pass(
        "A from B",
        &[(b, Usage::TEXTURE_BINDING)],
        &[(a, Usage::RENDER_ATTACHMENT)],
        |_| {},
    );
    graph.add_pass(
        "B from A",
        &[(a, Usage::TEXTURE_BINDING)],
        &[(b, Usage::RENDER_ATTACHMENT)],
        |_| {},
    );
    graph.export(b);
    assert!(graph.validate().unwrap_err().to_string().contains("cycle"));

    let mut graph = RenderGraph::new();
    let a = image("A", &mut graph);
    graph.add_pass(
        "Feedback",
        &[(a, Usage::STORAGE_BINDING)],
        &[(a, Usage::STORAGE_BINDING)],
        |_| {},
    );
    graph.export(a);
    assert!(
        graph
            .validate()
            .unwrap_err()
            .to_string()
            .contains("separate image versions")
    );
}

#[test]
fn requires_an_output_and_valid_access_directions() {
    let mut graph = RenderGraph::new();
    assert!(
        graph
            .validate()
            .unwrap_err()
            .to_string()
            .contains("no exported image")
    );
    let a = image("A", &mut graph);
    graph.add_pass("Invalid Write", &[], &[(a, Usage::TEXTURE_BINDING)], |_| {});
    graph.export(a);
    assert!(
        graph
            .validate()
            .unwrap_err()
            .to_string()
            .contains("invalid write usage")
    );
}

#[test]
fn relative_dimensions_round_down_and_keep_a_texel() {
    assert_eq!(
        ImageSize::Scaled {
            numerator: 1,
            denominator: 2
        }
        .resolve(67, 33)
        .unwrap(),
        (33, 16)
    );
    assert_eq!(
        ImageSize::Scaled {
            numerator: 1,
            denominator: 8
        }
        .resolve(3, 4)
        .unwrap(),
        (1, 1)
    );
    assert!(
        ImageSize::Scaled {
            numerator: 0,
            denominator: 2
        }
        .resolve(4, 4)
        .is_err()
    );
    assert!(
        ImageSize::Scaled {
            numerator: 1,
            denominator: 0
        }
        .resolve(4, 4)
        .is_err()
    );
    assert!(
        ImageSize::Scaled {
            numerator: u32::MAX,
            denominator: 1
        }
        .resolve(4, 4)
        .is_err()
    );
    assert!(
        ImageSize::Fixed {
            width: 0,
            height: 4
        }
        .resolve(4, 4)
        .is_err()
    );
}

#[test]
fn pass_cache_reuses_matching_state_replaces_changed_keys_and_drops_unused_state() {
    struct Resource(Rc<Cell<usize>>);
    impl Drop for Resource {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let drops = Rc::new(Cell::new(0));
    let mut cache = PassCache::default();
    cache.begin_render();
    cache.get_or_insert_with(1, || Resource(drops.clone()));
    cache.end_render();
    cache.begin_render();
    cache.get_or_insert_with::<_, Resource>(1, || panic!("Matching state must be reused"));
    cache.end_render();
    assert_eq!(drops.get(), 0);
    cache.begin_render();
    cache.get_or_insert_with(2, || Resource(drops.clone()));
    cache.end_render();
    assert_eq!(drops.get(), 1);
    cache.begin_render();
    cache.end_render();
    assert_eq!(drops.get(), 2);
}
