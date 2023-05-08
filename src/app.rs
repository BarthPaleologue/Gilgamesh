use winit::event_loop::EventLoop;
use winit::window::Window;
use crate::{Engine, Scene};

pub struct App {
    pub event_loop: EventLoop<()>,
    pub window: Window,
    pub engine: Engine,
    pub scene: Scene,
}