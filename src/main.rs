extern crate core;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowAttributes, WindowId};
use winit::{event::WindowEvent, event_loop::EventLoop};

use crate::renderer::Renderer;

mod buffer;
mod camera;
mod command_buffer;
mod command_buffer_helpers;
mod deferred_lightning_render_pass;
mod deferred_render_pass;
mod draw_data;
mod frame_worker;
mod image;
mod mesh;
mod pipeline;
mod pipeline_manager;
mod push_constants_data;
mod render_pass_attachment_output;
mod renderer;
mod shader_manager;
mod shadow_map_render_pass;
mod vertex;

#[derive(Default)]
struct State {
    window: Option<Window>,
    renderer: Option<Renderer>,
}

impl State {
    fn create_renderer(&mut self, window: &Window) {
        self.renderer = Some(Renderer::new(window));
    }
}

impl ApplicationHandler for State {
    // This is a common indicator that you can create a window.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(PhysicalSize::new(1920, 1080))
                        .with_resizable(false),
                )
                .ok();

            if let Some(window) = &window {
                self.create_renderer(window);
            }

            self.window = window;
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // `unwrap` is fine, the window will always be available when
        // receiving a window event.
        let _window = self.window.as_ref().unwrap();

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.render();
                }
                _window.request_redraw();
            }
            _ => (),
        }
    }

    // fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    //     let window = self.window.as_ref().unwrap();
    //     window.request_redraw();
    // }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut state = State::default();
    let _ = event_loop.run_app(&mut state);
}
