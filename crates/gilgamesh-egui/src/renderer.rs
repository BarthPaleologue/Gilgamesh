use crate::textures::TextureSnapshot;
use egui::TextureId;
use egui_wgpu::{Renderer, RendererOptions};
use gilgamesh::core::WGPUContext;
use std::collections::{HashMap, HashSet};
use wgpu::{Device, TextureFormat};

pub(super) struct GpuRenderer {
    pub renderer: Renderer,
    pub device: Device,
    pub format: TextureFormat,
    generation: u64,
    uploaded: HashSet<TextureId>,
}

impl GpuRenderer {
    pub(super) fn new(gpu: &WGPUContext, format: TextureFormat) -> Self {
        Self {
            renderer: Renderer::new(
                &gpu.device,
                format,
                RendererOptions {
                    dithering: false,
                    ..RendererOptions::default()
                },
            ),
            device: gpu.device.clone(),
            format,
            generation: 0,
            uploaded: HashSet::new(),
        }
    }

    pub(super) fn sync_textures(
        &mut self,
        gpu: &WGPUContext,
        textures: &HashMap<TextureId, TextureSnapshot>,
        generation: u64,
    ) {
        if self.generation == generation {
            return;
        }
        self.uploaded.retain(|id| {
            if textures.contains_key(id) {
                true
            } else {
                self.renderer.free_texture(id);
                false
            }
        });
        for (&id, texture) in textures {
            if texture.changed_at > self.generation {
                self.renderer
                    .update_texture(&gpu.device, &gpu.queue, id, &texture.image);
                self.uploaded.insert(id);
            }
        }
        self.generation = generation;
    }
}
