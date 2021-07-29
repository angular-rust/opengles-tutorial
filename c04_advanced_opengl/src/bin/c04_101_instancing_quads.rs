use c04_advanced_opengl::Shader;
use cgmath::Vector2;
use dx::{
    assets,
    gles::{core30::gl, enums::*},
    Instance,
};
use num::range_step;
use std::mem;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// settings
// const SCR_WIDTH: u32 = 1280;
// const SCR_HEIGHT: u32 = 720;

pub fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

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

    let (shader, quadvao) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);

        // build and compile shaders
        let shader = Shader::new(assets!("shaders/10.1.instancing.vs"), assets!("shaders/10.1.instancing.fs"));

        // generate a list of 100 quad locations/translation-vectors

        let mut translations = vec![];
        let offset = 0.1;
        for y in range_step(-10, 10, 2) {
            for x in range_step(-10, 10, 2) {
                translations.push(Vector2 {
                    x: x as i32 as f32 / 10.0 + offset,
                    y: y as i32 as f32 / 10.0 + offset,
                })
            }
        }

        let instancevbo = gl::gen_buffer();
        gl::bind_buffer(GL_ARRAY_BUFFER, instancevbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &translations, GL_STATIC_DRAW);
        gl::bind_buffer(GL_ARRAY_BUFFER, 0);

        // set up vertex data (and buffer(s)) and configure vertex attributes

        let quad_vertices: [f32; 30] = [
            // positions   // colors
            -0.05, 0.05, 1.0, 0.0, 0.0, 0.05, -0.05, 0.0, 1.0, 0.0, -0.05, -0.05, 0.0, 0.0, 1.0, -0.05, 0.05, 1.0, 0.0,
            0.0, 0.05, -0.05, 0.0, 1.0, 0.0, 0.05, 0.05, 0.0, 1.0, 1.0,
        ];

        let quadvao = gl::gen_vertex_array();
        let quadvbo = gl::gen_buffer();
        gl::bind_vertex_array(quadvao);
        gl::bind_buffer(GL_ARRAY_BUFFER, quadvbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &quad_vertices, GL_STATIC_DRAW);
        gl::enable_vertex_attrib_array(0);
        let stride = 5 * mem::size_of::<f32>() as i32;
        gl::vertex_attrib_pointer_offset(0, 2, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(1);
        gl::vertex_attrib_pointer_offset(1, 3, GL_FLOAT, false, stride, 2 * mem::size_of::<f32>() as u32);
        // also set instance data
        gl::enable_vertex_attrib_array(2);
        gl::bind_buffer(GL_ARRAY_BUFFER, instancevbo); // this attribute comes from a different vertex buffer
        gl::vertex_attrib_pointer_offset(2, 2, GL_FLOAT, false, (2 * mem::size_of::<f32>()) as i32, 0);
        gl::bind_buffer(GL_ARRAY_BUFFER, 0);
        gl::vertex_attrib_divisor(2, 1); // tell OpenGL this is an instanced vertex attribute.

        (shader, quadvao)
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
                _ => {}
            },
            WindowEvent::Resized(physical_size) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                gl::viewport(0, 0, physical_size.width as i32, physical_size.height as i32);
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            // redraw here for not active games like a RPG or RTS
            gl::clear_color(0.1, 0.1, 0.1, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            // draw 100 instanced quads
            shader.use_program();
            gl::bind_vertex_array(quadvao);
            gl::draw_arrays_instanced(GL_TRIANGLES, 0, 6, 100); // 100 triangles of 6 vertices each
            gl::bind_vertex_array(0);

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
    // gl::delete_vertex_arrays(1, &quadvao);
    // gl::delete_buffers(1, &quadvbo);
}
