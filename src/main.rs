use winit::dpi::PhysicalSize;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::renderer::Renderer;

mod pipeline;
mod renderer;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize {
            width: 1920,
            height: 1080,
        })
        .build(&event_loop)
        .unwrap();

    let renderer = Renderer::new(&window);

    event_loop.run(move |event, _, control_flow| {
        //control_flow.set_poll();

        match event {
            Event::NewEvents(_) => {}
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::CloseRequested = event {
                    control_flow.set_exit();
                }
            }
            Event::DeviceEvent { .. } => {}
            Event::UserEvent(_) => {}
            Event::Suspended => {}
            Event::Resumed => {}
            Event::MainEventsCleared => {}
            Event::RedrawRequested(_) => {
                renderer.render();
            }
            Event::RedrawEventsCleared => {}
            Event::LoopDestroyed => {}
        }
    });
}
