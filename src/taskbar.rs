
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{WindowBuilder, Window, Fullscreen};
use winit::platform::windows::WindowBuilderExtWindows; // Для Windows

fn create_taskbar_window(event_loop: &EventLoop<()>) -> Window {
    let window = WindowBuilder::new()
        .with_decorations(false)
        .with_resizable(false)
        .with_always_on_top(true)
        .build(event_loop)
        .unwrap();

    window.set_fullscreen(Some(Fullscreen::Borderless(window.primary_monitor())));

    window
}

fn main() {
    let event_loop = EventLoop::new();
    let window = create_taskbar_window(&event_loop);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            },
            _ => (),
        }
    });
}
