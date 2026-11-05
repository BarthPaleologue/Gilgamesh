//! Graph passes with automatic bindings, resource declarations and cached GPU state.
use super::{Image, PassImages, RenderGraph};
use crate::core::fullscreen_pass;
use std::borrow::Cow;
use wgpu::{TextureFormat, TextureUsages as Usage};

type UniformValue<'a> = Box<dyn FnOnce(&PassImages<'_>) -> Vec<u8> + 'a>;

/// Bindings occupy successive indices in WGSL group 0.
/// Uniform values must match WGSL's field layout; their buffer is padded to 16 bytes.
pub enum Binding<'a> {
    /// A sampled texture read by the shader.
    Texture(Image),
    /// A write-only storage texture. Its WGSL format must match the image format.
    StorageTexture(Image),
    Sampler(wgpu::SamplerDescriptor<'static>),
    #[doc(hidden)]
    Uniform(UniformValue<'a>),
}

impl<'a> Binding<'a> {
    pub fn uniform<T: bytemuck::Pod>(value: T) -> Self {
        let bytes = bytemuck::bytes_of(&value).to_vec();
        Self::Uniform(Box::new(move |_| bytes))
    }

    /// Resolves a uniform at execution, for example from an output's actual dimensions.
    /// Only images already declared by the pass can be queried.
    pub fn uniform_with<T: bytemuck::Pod>(value: impl FnOnce(&PassImages<'_>) -> T + 'a) -> Self {
        Self::Uniform(Box::new(move |images| {
            bytemuck::bytes_of(&value(images)).to_vec()
        }))
    }
}

/// Explicit workgroups or a 2D image domain.
#[must_use]
pub enum Dispatch {
    Workgroups([u32; 3]),
    Image {
        image: Image,
        workgroup_size: [u32; 2],
    },
}

impl Dispatch {
    /// # Panics
    /// Panics if either workgroup dimension is zero.
    pub fn for_image(image: Image, workgroup_size: [u32; 2]) -> Self {
        assert!(
            workgroup_size.into_iter().all(|n| n > 0),
            "Workgroup dimensions must be positive"
        );
        Self::Image {
            image,
            workgroup_size,
        }
    }

    fn resolve(self, images: &PassImages<'_>) -> [u32; 3] {
        match self {
            Self::Workgroups(groups) => groups,
            Self::Image {
                image,
                workgroup_size: [x, y],
            } => {
                let size = images.get(image).texture().size();
                [size.width.div_ceil(x), size.height.div_ceil(y), 1]
            }
        }
    }
}

/// Adds a fullscreen fragment pass. WGSL supplies `fs_main`; the vertex stage is shared.
/// Outputs correspond to fragment locations 0..N and must have matching dimensions.
/// The pass clears and replaces all outputs. Bindings must match the shader's group 0;
/// uniforms and samplers are cached, while texture views are resolved each execution.
///
/// # Panics
/// Panics if no output is supplied or an output belongs to another graph.
pub fn fragment_pass<'a>(
    label: impl Into<String>,
    shader: impl Into<Cow<'static, str>>,
    bindings: impl IntoIterator<Item = Binding<'a>>,
    outputs: &[Image],
    graph: &mut RenderGraph<'a>,
) {
    assert!(
        !outputs.is_empty(),
        "A fragment pass needs at least one output"
    );
    let bindings: Vec<_> = bindings.into_iter().collect();
    let (reads, mut writes) = accesses(&bindings);
    writes.extend(
        outputs
            .iter()
            .map(|&image| (image, Usage::RENDER_ATTACHMENT)),
    );
    let formats: Vec<_> = outputs
        .iter()
        .map(|&image| graph.image_descriptor(image).format)
        .collect();
    let outputs = outputs.to_vec();
    let shader = shader.into();
    graph.add_pass(label, &reads, &writes, move |context| {
        let values = resolve(bindings, &context.images);
        let key = Key::new(shader.clone(), formats.clone(), &values);
        let state = context.cache.get_or_insert_with(key, || State {
            pipeline: fullscreen_pass::create_pipeline(
                &context.gpu.device,
                &shader,
                "fs_main",
                &formats,
            ),
            resources: resources(&context.gpu.device, &values),
        });
        let group = bind(
            &context.gpu.device,
            &context.gpu.queue,
            &context.images,
            || state.pipeline.get_bind_group_layout(0),
            &state.resources,
            &values,
        );
        let targets: Vec<_> = outputs
            .iter()
            .map(|&image| context.images.get(image))
            .collect();
        fullscreen_pass::encode_targets(context.encoder, &state.pipeline, group.as_ref(), &targets);
    });
}

/// Adds a compute pass whose WGSL entry point is `main`.
/// `StorageTexture` bindings declare writes; use `next_version` before overwriting an image.
/// An image dispatch domain must also appear in the bindings. The shader must guard
/// out-of-bounds invocations when dimensions are not multiples of its workgroup size.
pub fn compute_pass<'a>(
    label: impl Into<String>,
    shader: impl Into<Cow<'static, str>>,
    bindings: impl IntoIterator<Item = Binding<'a>>,
    dispatch: Dispatch,
    graph: &mut RenderGraph<'a>,
) {
    let bindings: Vec<_> = bindings.into_iter().collect();
    let (reads, writes) = accesses(&bindings);
    let shader = shader.into();
    graph.add_pass(label, &reads, &writes, move |context| {
        let values = resolve(bindings, &context.images);
        let key = Key::new(shader.clone(), Vec::new(), &values);
        let state = context.cache.get_or_insert_with(key, || {
            let module = context
                .gpu
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Compute pass"),
                    source: wgpu::ShaderSource::Wgsl(shader.clone()),
                });
            State {
                pipeline: context.gpu.device.create_compute_pipeline(
                    &wgpu::ComputePipelineDescriptor {
                        label: Some("Compute pass"),
                        layout: None,
                        module: &module,
                        entry_point: Some("main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        cache: None,
                    },
                ),
                resources: resources(&context.gpu.device, &values),
            }
        });
        let groups = dispatch.resolve(&context.images);
        if groups.contains(&0) {
            return;
        }
        let group = bind(
            &context.gpu.device,
            &context.gpu.queue,
            &context.images,
            || state.pipeline.get_bind_group_layout(0),
            &state.resources,
            &values,
        );
        let mut pass = context
            .encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass.set_pipeline(&state.pipeline);
        if let Some(group) = &group {
            pass.set_bind_group(0, group, &[]);
        }
        pass.dispatch_workgroups(groups[0], groups[1], groups[2]);
    });
}

type Accesses = Vec<(Image, Usage)>;
fn accesses(bindings: &[Binding<'_>]) -> (Accesses, Accesses) {
    let mut reads = Vec::new();
    let mut writes = Vec::new();
    for binding in bindings {
        match binding {
            Binding::Texture(image) => reads.push((*image, Usage::TEXTURE_BINDING)),
            Binding::StorageTexture(image) => writes.push((*image, Usage::STORAGE_BINDING)),
            _ => {}
        }
    }
    (reads, writes)
}

enum Value {
    Image(Image),
    Uniform(Vec<u8>),
    Sampler(wgpu::SamplerDescriptor<'static>),
}

fn resolve(bindings: Vec<Binding<'_>>, images: &PassImages<'_>) -> Vec<Value> {
    bindings
        .into_iter()
        .map(|binding| match binding {
            Binding::Texture(image) | Binding::StorageTexture(image) => Value::Image(image),
            Binding::Sampler(descriptor) => Value::Sampler(descriptor),
            Binding::Uniform(value) => {
                let mut bytes = value(images);
                assert!(!bytes.is_empty(), "Uniforms must not be empty");
                bytes.resize(bytes.len().next_multiple_of(16), 0);
                Value::Uniform(bytes)
            }
        })
        .collect()
}

#[derive(PartialEq, Eq)]
struct Key {
    shader: Cow<'static, str>,
    formats: Vec<TextureFormat>,
    bindings: Vec<ResourceKey>,
}
#[derive(PartialEq, Eq)]
enum ResourceKey {
    Image,
    Uniform(usize),
    Sampler(SamplerKey),
}
struct SamplerKey(wgpu::SamplerDescriptor<'static>);
impl PartialEq for SamplerKey {
    fn eq(&self, other: &Self) -> bool {
        let (a, b) = (&self.0, &other.0);
        a.address_mode_u == b.address_mode_u
            && a.address_mode_v == b.address_mode_v
            && a.address_mode_w == b.address_mode_w
            && a.mag_filter == b.mag_filter
            && a.min_filter == b.min_filter
            && a.mipmap_filter == b.mipmap_filter
            && a.lod_min_clamp.to_bits() == b.lod_min_clamp.to_bits()
            && a.lod_max_clamp.to_bits() == b.lod_max_clamp.to_bits()
            && a.compare == b.compare
            && a.anisotropy_clamp == b.anisotropy_clamp
            && a.border_color == b.border_color
    }
}
impl Eq for SamplerKey {}
impl Key {
    fn new(shader: Cow<'static, str>, formats: Vec<TextureFormat>, values: &[Value]) -> Self {
        Self {
            shader,
            formats,
            bindings: values
                .iter()
                .map(|value| match value {
                    Value::Image(_) => ResourceKey::Image,
                    Value::Uniform(bytes) => ResourceKey::Uniform(bytes.len()),
                    Value::Sampler(descriptor) => {
                        ResourceKey::Sampler(SamplerKey(descriptor.clone()))
                    }
                })
                .collect(),
        }
    }
}

struct State<P> {
    pipeline: P,
    resources: Vec<Resource>,
}
enum Resource {
    Image,
    Uniform(wgpu::Buffer),
    Sampler(wgpu::Sampler),
}
fn resources(device: &wgpu::Device, values: &[Value]) -> Vec<Resource> {
    values
        .iter()
        .map(|value| match value {
            Value::Image(_) => Resource::Image,
            Value::Uniform(bytes) => {
                Resource::Uniform(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Pass uniform"),
                    size: bytes.len() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }))
            }
            Value::Sampler(descriptor) => Resource::Sampler(device.create_sampler(descriptor)),
        })
        .collect()
}
fn bind(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    images: &PassImages<'_>,
    layout: impl FnOnce() -> wgpu::BindGroupLayout,
    resources: &[Resource],
    values: &[Value],
) -> Option<wgpu::BindGroup> {
    if values.is_empty() {
        return None;
    }
    let layout = layout();
    let entries: Vec<_> = resources
        .iter()
        .zip(values)
        .enumerate()
        .map(|(binding, (resource, value))| {
            let resource = match (resource, value) {
                (Resource::Image, Value::Image(image)) => {
                    wgpu::BindingResource::TextureView(images.get(*image))
                }
                (Resource::Uniform(buffer), Value::Uniform(bytes)) => {
                    queue.write_buffer(buffer, 0, bytes);
                    buffer.as_entire_binding()
                }
                (Resource::Sampler(sampler), Value::Sampler(_)) => {
                    wgpu::BindingResource::Sampler(sampler)
                }
                _ => unreachable!("Cached bindings must match their key"),
            };
            wgpu::BindGroupEntry {
                binding: binding as u32,
                resource,
            }
        })
        .collect();
    Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Shader pass"),
        layout: &layout,
        entries: &entries,
    }))
}
