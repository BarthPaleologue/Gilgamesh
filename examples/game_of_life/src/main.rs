use gilgamesh::blocks::resize_nearest;
use gilgamesh::core::app::{App, FrameContext};
use gilgamesh::core::render_graph::{ImageSize, RenderGraph};
use gilgamesh::core::wgpu_context::WGPUContext;
use gilgamesh::core::window_runtime::{WindowContext, WindowRuntime};
use wgpu::{TextureFormat, TextureUsages};
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

mod evolve;
mod randomize;

const GRID_SIZE: u32 = 128;
const STEP_SECONDS: f64 = 0.1;

fn main() {
    WindowRuntime::<GameOfLife>::run(768, 768);
}

struct GameOfLife {
    states: [wgpu::Texture; 2],
    current: usize,
    step_due: bool,
    randomize_due: bool,
    reset_requested: bool,
    seed: u32,
    accumulated_seconds: f64,
    paused: bool,
}

impl App for GameOfLife {
    fn new(ctx: &WGPUContext) -> Self {
        assert!(
            ctx.device.limits().max_compute_workgroups_per_dimension > 0,
            "Game of Life requires compute shaders (WebGPU); WebGL is not supported"
        );
        let states = ["Life A", "Life B"].map(|label| {
            ctx.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: GRID_SIZE,
                    height: GRID_SIZE,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::STORAGE_BINDING
                    | TextureUsages::COPY_DST,
                view_formats: &[],
            })
        });
        Self {
            states,
            current: 0,
            step_due: false,
            randomize_due: false,
            reset_requested: true,
            seed: 0,
            accumulated_seconds: 0.0,
            paused: false,
        }
    }

    fn name(&self) -> &str {
        if self.paused {
            "Game of Life — paused — Space: resume, R: reset"
        } else {
            "Game of Life — Space: pause, R: reset"
        }
    }

    fn update_before_render(&mut self, frame: &FrameContext<'_>) {
        self.step_due = false;
        self.randomize_due = self.reset_requested;
        if self.reset_requested {
            self.reset_requested = false;
            self.seed = self.seed.wrapping_add(1);
            self.current = 0;
            self.accumulated_seconds = 0.0;
        }
        if self.paused || frame.delta_seconds == 0.0 {
            return;
        }
        self.accumulated_seconds += frame.delta_seconds;
        if self.accumulated_seconds >= STEP_SECONDS {
            self.accumulated_seconds -= STEP_SECONDS;
            self.current = 1 - self.current;
            self.step_due = true;
        }
    }

    fn render_graph(&self) -> RenderGraph<'_> {
        let mut graph = RenderGraph::new();
        let mut states = self
            .states
            .each_ref()
            .map(|texture| graph.import_texture("Life state", texture));
        let previous = 1 - self.current;
        if self.randomize_due {
            let initial = if self.step_due {
                previous
            } else {
                self.current
            };
            states[initial] = randomize::randomize(states[initial], self.seed, &mut graph);
        }
        if self.step_due {
            states[self.current] =
                evolve::evolve(states[previous], states[self.current], &mut graph);
        }
        let image = resize_nearest(states[self.current], ImageSize::Target, &mut graph);
        graph.export(image);
        graph
    }

    fn on_window_event(&mut self, event: &WindowEvent, _context: &mut WindowContext<'_>) {
        if let WindowEvent::KeyboardInput { event, .. } = event
            && event.state == ElementState::Pressed
            && !event.repeat
        {
            match event.physical_key {
                PhysicalKey::Code(KeyCode::Space) => self.paused = !self.paused,
                PhysicalKey::Code(KeyCode::KeyR) => self.reset_requested = true,
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests;
