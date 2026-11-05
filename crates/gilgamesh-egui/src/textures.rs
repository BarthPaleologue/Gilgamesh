use egui::epaint::ImageDelta;
use egui::{ImageData, TextureId};
use std::collections::HashMap;
use std::sync::Arc;

pub(super) struct TextureSnapshot {
    pub image: ImageDelta,
    pub changed_at: u64,
}

/// Keep current pixels so a renderer can be rebuilt for another device or target format.
pub(super) fn update(
    textures: &mut HashMap<TextureId, TextureSnapshot>,
    id: TextureId,
    delta: ImageDelta,
    generation: u64,
) {
    if let Some([x, y]) = delta.pos {
        let texture = textures
            .get_mut(&id)
            .expect("egui partial update needs an allocated texture");
        let ImageData::Color(destination) = &mut texture.image.image;
        let ImageData::Color(source) = delta.image;
        let destination = Arc::make_mut(destination);
        for (row, pixels) in source.pixels.chunks_exact(source.size[0]).enumerate() {
            let offset = (y + row) * destination.size[0] + x;
            destination.pixels[offset..offset + pixels.len()].copy_from_slice(pixels);
        }
        texture.image.options = delta.options;
        texture.changed_at = generation;
    } else {
        textures.insert(
            id,
            TextureSnapshot {
                image: delta,
                changed_at: generation,
            },
        );
    }
}
