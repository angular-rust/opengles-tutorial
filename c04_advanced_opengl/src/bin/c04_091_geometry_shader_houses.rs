use c04_advanced_opengl::{process_events, process_input, Camera, Shader};
use cgmath::Point3;
use dx::{
    assets,
    gles::{core30::gl, enums::*},
    utils::*,
    Instance,
};
use std::{mem, time::SystemTime};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// settings
const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

pub fn main() {
    let start_time = SystemTime::now();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut camera = Camera {
        position: Point3::new(0.0, 0.0, 3.0),
        ..Camera::default()
    };

    let mut first_mouse = true;
    let mut last_x: f32 = SCR_WIDTH as f32 / 2.0;
    let mut last_y: f32 = SCR_HEIGHT as f32 / 2.0;

    // timing
    let mut last_frame: f32 = 0.0;

    // initialize and configure
    let instance = Instance::new();
    let surface = instance.create_surface(&window);
    let adapter = instance.request_adapter();
    let (device, _) = adapter.request_device();
    let desc = dx::SwapChainDescriptor {
        usage: dx::TextureUsage::RENDER_ATTACHMENT,
        format: dx::TextureFormat::Bgra8UnormSrgb,
        width: 800,
        height: 600,
        present_mode: dx::PresentMode::Fifo,
    };
    let swapchain = device.create_swap_chain(&surface, &desc);

    // tell to capture our mouse
    // window.set_cursor_mode(CursorMode::Disabled);

    let (shader, vao) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);

        // build and compile shaders
        let shader = Shader::with_geometry_shader(
            assets!("shaders/9.1.geometry_shader.vs"),
            assets!("shaders/9.1.geometry_shader.fs"),
            assets!("shaders/9.1.geometry_shader.gs"),
        );

        // set up vertex data (and buffer(s)) and configure vertex attributes

        let points: [f32; 20] = [
            -0.5, 0.5, 1.0, 0.0, 0.0, // top-left
            0.5, 0.5, 0.0, 1.0, 0.0, // top-right
            0.5, -0.5, 0.0, 0.0, 1.0, // bottom-right
            -0.5, -0.5, 1.0, 1.0, 0.0, // bottom-left
        ];

        // cube vao
        let vao = gl::gen_vertex_array();
        let vbo = gl::gen_buffer();
        gl::bind_vertex_array(vao);
        gl::bind_buffer(GL_ARRAY_BUFFER, vbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &points, GL_STATIC_DRAW);
        let stride = 5 * mem::size_of::<f32>() as i32;
        gl::enable_vertex_attrib_array(0);
        gl::vertex_attrib_pointer_offset(0, 2, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(1);
        gl::vertex_attrib_pointer_offset(1, 3, GL_FLOAT, false, stride, 2 * mem::size_of::<f32>() as u32);
        gl::bind_vertex_array(0);

        (shader, vao)
    };

    // render loop
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { ref event, window_id } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {
                    let current_frame = get_time(&start_time);

                    // input
                    process_input(&input, current_frame - last_frame, &mut camera);
                    last_frame = current_frame;
                }
            },
            WindowEvent::Resized(physical_size) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                gl::viewport(0, 0, physical_size.width as i32, physical_size.height as i32);
            }
            _ => {
                // events
                process_events(&event, &mut first_mouse, &mut last_x, &mut last_y, &mut camera);
            }
        },
        Event::MainEventsCleared => {
            // redraw here for not active games like a RPG or RTS
            // per-frame time logic

            gl::clear_color(0.1, 0.1, 0.1, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            shader.use_program();
            gl::bind_vertex_array(vao);
            gl::draw_arrays(GL_POINTS, 0, 4);

            // swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            swapchain.present(&surface);
            // request redraw again
            window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            // redraw here when something changed
        }
        _ => {}
    });

    // optional: de-allocate all resources once they've outlived their purpose:
    //     gl::delete_vertex_arrays(1, &vao);
    //     gl::delete_buffers(1, &vbo);
}
