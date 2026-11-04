use super::resize_bilinear;
use crate::core::render_graph::{Image, ImageSize, RenderGraph};

mod downsample;
mod merge_levels;
use downsample::downsample;
use merge_levels::merge_levels;

/// Blurs RGBA using a normalized pyramid of `levels` half-resolution images.
/// Uses a 13-tap reduction filter and bilinear reconstruction, blending levels equally.
/// Preserves the source size and format; uniform colors stay unchanged.
/// Requires a filterable color format. Zero levels returns the source unchanged.
pub fn pyramid_blur(source: Image, levels: u32, graph: &mut RenderGraph<'_>) -> Image {
    if levels == 0 {
        return source;
    }
    let mut pyramid = Vec::new();
    let mut image = source;
    for _ in 0..levels {
        let size = half_size(graph.image_descriptor(image).size);
        image = downsample(image, size, graph);
        pyramid.push(image);
        if matches!(
            size,
            ImageSize::Fixed {
                width: 1,
                height: 1
            }
        ) {
            break;
        }
    }
    let mut blurred = pyramid.pop().unwrap();
    for image in pyramid.into_iter().rev() {
        let expanded = resize_bilinear(blurred, graph.image_descriptor(image).size, graph);
        blurred = merge_levels(image, expanded, graph);
    }
    resize_bilinear(blurred, graph.image_descriptor(source).size, graph)
}

fn half_size(size: ImageSize) -> ImageSize {
    match size {
        ImageSize::Fixed { width, height } => ImageSize::Fixed {
            width: (width / 2).max(1),
            height: (height / 2).max(1),
        },
        ImageSize::Target => ImageSize::Scaled {
            numerator: 1,
            denominator: 2,
        },
        ImageSize::Scaled {
            numerator,
            denominator,
        } => {
            if numerator % 2 == 0 {
                ImageSize::Scaled {
                    numerator: numerator / 2,
                    denominator,
                }
            } else {
                ImageSize::Scaled {
                    numerator,
                    denominator: denominator
                        .checked_mul(2)
                        .expect("Blur pyramid scale overflows"),
                }
            }
        }
    }
}
