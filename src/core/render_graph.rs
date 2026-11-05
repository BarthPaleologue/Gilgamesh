//! Per-render descriptions of passes over single-sample, single-mip 2D images.
//!
//! Blocks are ordinary functions which add passes and return image handles. Passes
//! declare all graph images they access; their callbacks may borrow application data.

use crate::core::wgpu_context::WGPUContext;
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet, VecDeque, hash_map::Entry};
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

mod shader_pass;
pub use shader_pass::{Binding, Dispatch, compute_pass, fragment_pass};

pub use wgpu::{TextureFormat, TextureUsages};

static NEXT_GRAPH_ID: AtomicUsize = AtomicUsize::new(1);

/// Identifies an image within the graph that created it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Image {
    pub(crate) graph: usize,
    pub(crate) index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageSize {
    Target,
    Fixed {
        width: u32,
        height: u32,
    },
    /// Scales both target dimensions, rounding down and keeping at least one texel.
    Scaled {
        numerator: u32,
        denominator: u32,
    },
}

impl ImageSize {
    pub(crate) fn resolve(self, width: u32, height: u32) -> Result<(u32, u32), GraphError> {
        let size = match self {
            Self::Target => (width, height),
            Self::Fixed { width, height } => (width, height),
            Self::Scaled {
                numerator,
                denominator,
            } => {
                if numerator == 0 || denominator == 0 {
                    return Err(GraphError::new("Image scale must be positive"));
                }
                let scale = |dimension: u32| {
                    u32::try_from(
                        (u64::from(dimension) * u64::from(numerator) / u64::from(denominator))
                            .max(1),
                    )
                    .map_err(|_| GraphError::new("Scaled image dimensions overflow"))
                };
                (scale(width)?, scale(height)?)
            }
        };
        if size.0 == 0 || size.1 == 0 {
            return Err(GraphError::new("Image dimensions must be nonzero"));
        }
        Ok(size)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct ImageDescriptor {
    pub size: ImageSize,
    pub format: wgpu::TextureFormat,
}

#[derive(Clone)]
pub(crate) struct GraphImage<'a> {
    pub label: String,
    pub descriptor: ImageDescriptor,
    pub allocation: usize,
    pub previous: Option<usize>,
    pub imported: Option<&'a wgpu::Texture>,
}

type EncodePass<'a> = Box<dyn FnOnce(&mut PassContext<'_>) + 'a>;

pub(crate) struct GraphPass<'a> {
    pub label: String,
    pub reads: Vec<(Image, wgpu::TextureUsages)>,
    pub writes: Vec<(Image, wgpu::TextureUsages)>,
    pub encode: EncodePass<'a>,
}

/// A fresh description for one render. GPU resources persist in `GraphExecutor`.
pub struct RenderGraph<'a> {
    pub(crate) id: usize,
    pub(crate) images: Vec<GraphImage<'a>>,
    pub(crate) passes: Vec<GraphPass<'a>>,
    output: Option<Image>,
}

impl<'a> RenderGraph<'a> {
    pub fn new() -> Self {
        Self {
            id: NEXT_GRAPH_ID.fetch_add(1, Ordering::Relaxed),
            images: Vec::new(),
            passes: Vec::new(),
            output: None,
        }
    }

    /// Creates a transient image. Its contents are unspecified until written.
    pub fn create_image(&mut self, label: impl Into<String>, descriptor: ImageDescriptor) -> Image {
        let image = Image {
            graph: self.id,
            index: self.images.len(),
        };
        self.images.push(GraphImage {
            label: label.into(),
            descriptor,
            allocation: image.index,
            previous: None,
            imported: None,
        });
        image
    }

    /// Imports a single-mip, single-layer, single-sample 2D texture owned by the caller.
    /// Passes may read or write it according to the texture's creation usages.
    /// Reads depend on its writer, if any; otherwise they see its existing contents.
    /// Importing the same texture again returns its initial handle, not its latest version.
    ///
    /// # Panics
    /// Panics unless the texture is 2D, single-layer, single-mip and single-sample.
    pub fn import_texture(
        &mut self,
        label: impl Into<String>,
        texture: &'a wgpu::Texture,
    ) -> Image {
        if let Some(index) = self
            .images
            .iter()
            .position(|image| image.imported == Some(texture))
        {
            return Image {
                graph: self.id,
                index,
            };
        }
        assert!(
            texture.dimension() == wgpu::TextureDimension::D2
                && texture.depth_or_array_layers() == 1
                && texture.mip_level_count() == 1
                && texture.sample_count() == 1,
            "Graph imports require single-mip, single-layer, single-sample 2D textures"
        );
        let image = self.create_image(
            label,
            ImageDescriptor {
                size: ImageSize::Fixed {
                    width: texture.width(),
                    height: texture.height(),
                },
                format: texture.format(),
            },
        );
        self.images[image.index].imported = Some(texture);
        image
    }

    /// Creates a new logical version backed by the same physical texture.
    /// Its writer waits for the previous writer and every reader of the old version.
    /// Each version may have only one successor. Versions cannot be read and written
    /// together in one pass; use separate textures for a compute ping-pong step.
    ///
    /// # Panics
    /// Panics if the image belongs to another graph or already has a successor.
    pub fn next_version(&mut self, image: Image) -> Image {
        self.check_image(image).expect("Invalid image version");
        assert!(
            !self
                .images
                .iter()
                .any(|version| version.previous == Some(image.index)),
            "Image version already has a successor"
        );
        let mut version = self.images[image.index].clone();
        version.previous = Some(image.index);
        let next = Image {
            graph: self.id,
            index: self.images.len(),
        };
        self.images.push(version);
        next
    }

    /// # Panics
    /// Panics if the image belongs to another graph.
    pub fn image_descriptor(&self, image: Image) -> ImageDescriptor {
        assert_eq!(image.graph, self.id, "Image belongs to another graph");
        self.images[image.index].descriptor
    }

    /// Adds a custom raster, compute, or copy pass. Each logical version has at most one writer;
    /// created images and successor versions require one; initial imports may retain their contents.
    /// Usages describe how the callback accesses each image; execution order follows
    /// the producers of its reads, independently of pass registration order.
    pub fn add_pass(
        &mut self,
        label: impl Into<String>,
        reads: &[(Image, wgpu::TextureUsages)],
        writes: &[(Image, wgpu::TextureUsages)],
        encode: impl FnOnce(&mut PassContext<'_>) + 'a,
    ) {
        self.passes.push(GraphPass {
            label: label.into(),
            reads: reads.to_vec(),
            writes: writes.to_vec(),
            encode: Box::new(encode),
        });
    }

    /// Selects a graph-created image written into the caller's target.
    /// It must match the target's dimensions; imported textures remain caller-owned.
    pub fn export(&mut self, image: Image) {
        self.output = Some(image);
    }

    /// Checks resource access and dependencies without creating GPU resources.
    ///
    /// # Errors
    /// Returns an error for invalid image handles, resource usages, writers, exports,
    /// or cyclic pass dependencies.
    pub fn validate(&self) -> Result<(), GraphError> {
        self.compile().map(|_| ())
    }

    pub(crate) fn compile(&self) -> Result<GraphPlan, GraphError> {
        let output = self
            .output
            .ok_or_else(|| GraphError::new("Graph has no exported image"))?;
        self.check_image(output)?;
        if self.images[output.index].imported.is_some() {
            return Err(GraphError::new(
                "Export a graph-created image, not an imported texture",
            ));
        }
        if self
            .images
            .iter()
            .any(|image| image.previous == Some(output.index))
        {
            return Err(GraphError::new(
                "Export the final image version, or copy an older version before overwriting it",
            ));
        }
        let (producers, usages) = self.validate_accesses(output)?;
        let mut successors = vec![Vec::new(); self.passes.len()];
        let mut pending = vec![0; self.passes.len()];
        let mut depend = |before: usize, after: usize| {
            if !successors[before].contains(&after) {
                successors[before].push(after);
                pending[after] += 1;
            }
        };
        for (index, pass) in self.passes.iter().enumerate() {
            for &(image, _) in &pass.reads {
                if let Some(producer) = producers[image.index] {
                    depend(producer, index);
                }
            }
        }
        for (index, image) in self.images.iter().enumerate() {
            if let Some(previous) = image.previous {
                let writer = producers[index].expect("Successor versions require a writer");
                if let Some(producer) = producers[previous] {
                    depend(producer, writer);
                }
                for (reader, pass) in self.passes.iter().enumerate() {
                    if pass.reads.iter().any(|&(image, _)| image.index == previous) {
                        depend(reader, writer);
                    }
                }
            }
        }
        let mut ready: VecDeque<_> = pending
            .iter()
            .enumerate()
            .filter_map(|(index, &count)| (count == 0).then_some(index))
            .collect();
        let mut order = Vec::with_capacity(self.passes.len());
        while let Some(index) = ready.pop_front() {
            order.push(index);
            for &next in &successors[index] {
                pending[next] -= 1;
                if pending[next] == 0 {
                    ready.push_back(next);
                }
            }
        }
        if order.len() != self.passes.len() {
            return Err(GraphError::new("Graph contains a dependency cycle"));
        }
        let mut lifetimes = vec![None; self.images.len()];
        for (position, &pass) in order.iter().enumerate() {
            for &(image, _) in self.passes[pass]
                .reads
                .iter()
                .chain(&self.passes[pass].writes)
            {
                let allocation = self.images[image.index].allocation;
                let lifetime = lifetimes[allocation].get_or_insert((position, position));
                lifetime.1 = position;
            }
        }
        Ok(GraphPlan {
            lifetimes,
            output,
            usages,
            order,
        })
    }

    fn validate_accesses(
        &self,
        output: Image,
    ) -> Result<(Vec<Option<usize>>, Vec<wgpu::TextureUsages>), GraphError> {
        let mut producers = vec![None; self.images.len()];
        let mut usages = vec![wgpu::TextureUsages::empty(); self.images.len()];
        for (index, pass) in self.passes.iter().enumerate() {
            let mut written_allocations = HashSet::new();
            for &(image, usage) in &pass.writes {
                self.check_image(image)?;
                let resource = &self.images[image.index];
                if !written_allocations.insert(resource.allocation) {
                    return Err(GraphError::new(
                        "A pass cannot write multiple versions of the same texture",
                    ));
                }
                let allowed = wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::COPY_DST;
                if usage.is_empty() || !allowed.contains(usage) {
                    return Err(GraphError::new(format!(
                        "{}: invalid write usage for {}",
                        pass.label, resource.label
                    )));
                }
                if producers[image.index].replace(index).is_some() {
                    return Err(GraphError::new(format!(
                        "Image {} has multiple writers",
                        resource.label
                    )));
                }
                usages[image.index] |= usage;
            }
            for &(image, usage) in &pass.reads {
                self.check_image(image)?;
                let allowed = wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::COPY_SRC;
                if usage.is_empty() || !allowed.contains(usage) {
                    return Err(GraphError::new(format!(
                        "{}: invalid read usage for {}",
                        pass.label, self.images[image.index].label
                    )));
                }
                if pass.writes.iter().any(|&(written, _)| {
                    self.images[written.index].allocation == self.images[image.index].allocation
                }) {
                    return Err(GraphError::new(format!(
                        "{}: a pass cannot read and write the same physical texture",
                        pass.label
                    )));
                }
                usages[image.index] |= usage;
            }
        }
        usages[output.index] |=
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC;
        let mut physical_usages = vec![wgpu::TextureUsages::empty(); self.images.len()];
        for (image, &usage) in self.images.iter().zip(&usages) {
            physical_usages[image.allocation] |= usage;
        }
        for (index, image) in self.images.iter().enumerate() {
            usages[index] = physical_usages[image.allocation];
            if image.previous.is_some() && producers[index].is_none() {
                return Err(GraphError::new(format!(
                    "Image version {} has no writer",
                    image.label
                )));
            }
            if let Some(texture) = image.imported {
                if !texture.usage().contains(usages[index]) {
                    return Err(GraphError::new(format!(
                        "Imported image {} lacks required usages",
                        image.label
                    )));
                }
            } else if producers[index].is_none() {
                return Err(GraphError::new(format!(
                    "Image {} has no writer",
                    image.label
                )));
            }
        }
        Ok((producers, usages))
    }

    fn check_image(&self, image: Image) -> Result<(), GraphError> {
        if image.graph != self.id || image.index >= self.images.len() {
            return Err(GraphError::new("Image belongs to another graph"));
        }
        Ok(())
    }
}

impl Default for RenderGraph<'_> {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct GraphPlan {
    pub lifetimes: Vec<Option<(usize, usize)>>,
    pub output: Image,
    pub usages: Vec<wgpu::TextureUsages>,
    pub order: Vec<usize>,
}

/// GPU services for one pass. Images are restricted to its declared accesses.
pub struct PassContext<'a> {
    pub gpu: &'a WGPUContext,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub images: PassImages<'a>,
    pub cache: &'a mut PassCache,
}

pub struct PassImages<'a> {
    pub(crate) graph: usize,
    pub(crate) views: &'a [wgpu::TextureView],
    pub(crate) reads: &'a [(Image, wgpu::TextureUsages)],
    pub(crate) writes: &'a [(Image, wgpu::TextureUsages)],
}

impl PassImages<'_> {
    /// # Panics
    /// Panics if the image does not belong to this graph or was not declared by this pass.
    #[must_use]
    pub fn get(&self, image: Image) -> &wgpu::TextureView {
        assert!(
            image.graph == self.graph
                && self
                    .reads
                    .iter()
                    .chain(self.writes)
                    .any(|&(declared, _)| declared == image),
            "Pass accessed an image it did not declare"
        );
        &self.views[image.index]
    }
}

/// GPU state retained per pass between executions. Keys must include everything
/// that affects construction. Bindings to graph images should be rebuilt per render.
#[derive(Default)]
pub struct PassCache {
    entries: HashMap<TypeId, CacheEntry>,
}

struct CacheEntry {
    used: bool,
    value: Box<dyn Any>,
}

struct Cached<K, V> {
    key: K,
    value: V,
}

impl PassCache {
    /// Retains one value per `(K, V)` type pair; replaces it when the key changes.
    #[expect(
        clippy::missing_panics_doc,
        reason = "Cache entries are indexed by their exact key and value types"
    )]
    pub fn get_or_insert_with<K: Eq + 'static, V: 'static>(
        &mut self,
        key: K,
        create: impl FnOnce() -> V,
    ) -> &mut V {
        let entry = match self.entries.entry(TypeId::of::<(K, V)>()) {
            Entry::Vacant(entry) => entry.insert(CacheEntry {
                used: true,
                value: Box::new(Cached {
                    key,
                    value: create(),
                }),
            }),
            Entry::Occupied(entry) => {
                let entry = entry.into_mut();
                let cached = entry.value.downcast_mut::<Cached<K, V>>().unwrap();
                if cached.key != key {
                    *cached = Cached {
                        key,
                        value: create(),
                    };
                }
                entry
            }
        };
        entry.used = true;
        &mut entry.value.downcast_mut::<Cached<K, V>>().unwrap().value
    }

    pub(crate) fn begin_render(&mut self) {
        for entry in self.entries.values_mut() {
            entry.used = false;
        }
    }

    pub(crate) fn end_render(&mut self) {
        self.entries.retain(|_, entry| entry.used);
    }
}

#[derive(Debug)]
pub struct GraphError(String);

impl GraphError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for GraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for GraphError {}

#[cfg(test)]
mod tests;
