use winit::{
    dpi::{LogicalSize, PhysicalSize, Size},
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();

    let wb = WindowBuilder::new()
        .with_min_inner_size(Size::Logical(LogicalSize::new(64.0, 64.0)))
        .with_inner_size(Size::Physical(PhysicalSize::new(900, 700)))
        .with_title("Game".to_string());

    let _window = wb.build(&event_loop).unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(_) => {
                    // make changes based on window size
                }
                _ => {}
            },
            Event::RedrawEventsCleared => {
                // render window contents here
            }
            _ => {}
        }
    });
}
