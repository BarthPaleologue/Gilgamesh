use super::*;
use std::cell::Cell;
use std::rc::Rc;
use wgpu::TextureUsages as Usage;

#[test]
fn blocks_compose_in_nested_calls_without_a_gpu() {
    use crate::blocks::{render_scene, resize_nearest};
    use crate::camera::Camera;
    use crate::core::render_context::FrameBindingsData;
    use crate::lights::directional_light::DirectionalLight;

    let camera: Camera = Camera::default();
    let light = DirectionalLight::default();
    let mut graph = RenderGraph::new();
    let output = resize_nearest(
        render_scene(
            FrameBindingsData::new(&camera, std::slice::from_ref(&light), &[]),
            [],
            [0.1, 0.1, 0.2, 1.0],
            &mut graph,
        ),
        ImageSize::Target,
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
            .contains("same physical texture")
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

#[test]
fn versions_wait_for_old_readers_and_reuse_one_allocation() {
    let mut graph = RenderGraph::new();
    let a = image("A", &mut graph);
    let b = image("B", &mut graph);
    let next = graph.next_version(a);
    let snapshot = image("Snapshot", &mut graph);
    graph.add_pass(
        "Overwrite A",
        &[(b, Usage::COPY_SRC)],
        &[(next, Usage::COPY_DST)],
        |_| {},
    );
    graph.add_pass(
        "Old reader",
        &[(a, Usage::TEXTURE_BINDING)],
        &[(snapshot, Usage::RENDER_ATTACHMENT)],
        |_| {},
    );
    graph.add_pass(
        "A to B",
        &[(a, Usage::COPY_SRC)],
        &[(b, Usage::COPY_DST)],
        |_| {},
    );
    graph.add_pass(
        "Initialize A",
        &[],
        &[(a, Usage::RENDER_ATTACHMENT)],
        |_| {},
    );
    graph.export(next);
    let plan = graph.compile().unwrap();
    assert_eq!(plan.order, [3, 1, 2, 0]);
    assert_eq!(
        graph.images[a.index].allocation,
        graph.images[next.index].allocation
    );
    assert_eq!(plan.usages[a.index], plan.usages[next.index]);
    assert!(plan.usages[a.index].contains(
        Usage::COPY_DST | Usage::COPY_SRC | Usage::TEXTURE_BINDING | Usage::RENDER_ATTACHMENT
    ));
}

#[test]
fn versions_require_a_writer_and_reject_physical_feedback_and_old_exports() {
    let mut graph = RenderGraph::new();
    let a = image("A", &mut graph);
    let next = graph.next_version(a);
    graph.add_pass("Initialize", &[], &[(a, Usage::RENDER_ATTACHMENT)], |_| {});
    graph.export(next);
    assert!(
        graph
            .validate()
            .unwrap_err()
            .to_string()
            .contains("has no writer")
    );
    graph.add_pass("Overwrite", &[], &[(next, Usage::COPY_DST)], |_| {});
    graph.export(a);
    assert!(
        graph
            .validate()
            .unwrap_err()
            .to_string()
            .contains("final image version")
    );
    graph.export(next);
    graph.validate().unwrap();
    graph.passes[1].reads.push((a, Usage::COPY_SRC));
    assert!(
        graph
            .validate()
            .unwrap_err()
            .to_string()
            .contains("same physical texture")
    );
}

#[test]
#[should_panic(expected = "already has a successor")]
fn versions_cannot_branch_over_the_same_physical_texture() {
    let mut graph = RenderGraph::new();
    let a = image("A", &mut graph);
    graph.next_version(a);
    graph.next_version(a);
}

#[test]
fn rejects_an_old_version_read_that_depends_on_its_overwrite() {
    let mut graph = RenderGraph::new();
    let initial = image("A", &mut graph);
    let next = graph.next_version(initial);
    let output = image("Impossible snapshot", &mut graph);
    graph.add_pass(
        "Initialize",
        &[],
        &[(initial, Usage::RENDER_ATTACHMENT)],
        |_| {},
    );
    graph.add_pass(
        "Overwrite",
        &[],
        &[(next, Usage::RENDER_ATTACHMENT)],
        |_| {},
    );
    graph.add_pass(
        "Read both versions",
        &[
            (initial, Usage::TEXTURE_BINDING),
            (next, Usage::TEXTURE_BINDING),
        ],
        &[(output, Usage::RENDER_ATTACHMENT)],
        |_| {},
    );
    graph.export(output);
    assert!(graph.validate().unwrap_err().to_string().contains("cycle"));
}
