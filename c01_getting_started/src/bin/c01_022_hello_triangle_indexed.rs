use dx::{
    gles::{core30::gl, enums::*},
    Instance,
};
use std::{mem, str};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// settings
// const SCR_WIDTH: u32 = 800;
// const SCR_HEIGHT: u32 = 600;

const VERTEX_SHADER_SOURCE: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    void main() {
       gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    #version 330 core
    out vec4 FragColor;
    void main() {
       FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    }
"#;

pub fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // instance param is - backend type.
    // mean Primary supported types like VK, Metal, DX12 WebGPU
    // Secondary is OpenGL or DX11

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

    let (shader_program, vao) = {
        // build and compile our shader program

        // vertex shader
        let vertex_shader = gl::create_shader(GL_VERTEX_SHADER);
        gl::shader_source(vertex_shader, VERTEX_SHADER_SOURCE.as_bytes());
        gl::compile_shader(vertex_shader);

        // check for shader compile errors
        let success = gl::get_shaderiv(vertex_shader, GL_COMPILE_STATUS);
        if success == 0 {
            let len = gl::get_shaderiv(vertex_shader, GL_INFO_LOG_LENGTH);

            return match gl::get_shader_info_log(vertex_shader, len) {
                Some(message) => {
                    panic!("ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}", message);
                }
                None => {}
            };
        }

        // fragment shader
        let fragment_shader = gl::create_shader(GL_FRAGMENT_SHADER);
        gl::shader_source(fragment_shader, FRAGMENT_SHADER_SOURCE.as_bytes());
        gl::compile_shader(fragment_shader);
        // check for shader compile errors
        let success = gl::get_shaderiv(fragment_shader, GL_COMPILE_STATUS);
        if success == 0 {
            let len = gl::get_shaderiv(fragment_shader, GL_INFO_LOG_LENGTH);

            return match gl::get_shader_info_log(fragment_shader, len) {
                Some(message) => {
                    panic!("ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}", message);
                }
                None => {}
            };
        }

        // link shaders
        let shader_program = gl::create_program();
        gl::attach_shader(shader_program, vertex_shader);
        gl::attach_shader(shader_program, fragment_shader);
        gl::link_program(shader_program);
        // check for linking errors
        let success = gl::get_programiv(shader_program, GL_LINK_STATUS);

        if success == 0 {
            let len = gl::get_programiv(shader_program, GL_INFO_LOG_LENGTH);

            return match gl::get_program_info_log(shader_program, len) {
                Some(message) => {
                    println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", message);
                }
                None => {}
            };
        }

        gl::delete_shader(vertex_shader);
        gl::delete_shader(fragment_shader);

        // set up vertex data (and buffer(s)) and configure vertex attributes

        // HINT: type annotation is crucial since default for float literals is f64
        let vertices: [f32; 12] = [
            0.5, 0.5, 0.0, // top right
            0.5, -0.5, 0.0, // bottom right
            -0.5, -0.5, 0.0, // bottom left
            -0.5, 0.5, 0.0, // top left
        ];
        let indices = [
            // note that we start from 0!
            0_u32, 1, 3, // first Triangle
            1, 2, 3, // second Triangle
        ];

        let vao = gl::gen_vertex_array();
        let vbo = gl::gen_buffer();
        let ebo = gl::gen_buffer();
        // bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attributes(s).
        gl::bind_vertex_array(vao);

        gl::bind_buffer(GL_ARRAY_BUFFER, vbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &vertices, GL_STATIC_DRAW);

        gl::bind_buffer(GL_ELEMENT_ARRAY_BUFFER, ebo);
        gl::buffer_data(GL_ELEMENT_ARRAY_BUFFER, &indices, GL_STATIC_DRAW);

        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, 3 * mem::size_of::<f32>() as i32, 0);
        gl::enable_vertex_attrib_array(0);

        // note that this is allowed, the call to gl::vertex_attrib_pointer registered vbo as the vertex attribute's bound vertex buffer object so afterwards we can safely unbind
        gl::bind_buffer(GL_ARRAY_BUFFER, 0);

        // remember: do NOT unbind the EBO while a vao is active as the bound element buffer object IS stored in the vao; keep the EBO bound.
        // gl::bind_buffer(GL_ELEMENT_ARRAY_BUFFER, 0);

        // You can unbind the vao afterwards so other vao calls won't accidentally modify this vao, but this rarely happens. Modifying other
        // vaos requires a call to gl::bind_vertex_array anyways so we generally don't unbind vaos (nor vbos) when it's not directly necessary.
        gl::bind_vertex_array(0);

        // uncomment this call to draw in wireframe polygons.
        // gl::PolygonMode(GL_FRONT_AND_BACK, GL_LINE);

        (shader_program, vao)
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
            gl::clear_color(0.2, 0.3, 0.3, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT);

            // draw our first triangle
            gl::use_program(shader_program);
            // seeing as we only have a single vao there's no need to bind it every time, but we'll do so to keep things a bit more organized
            gl::bind_vertex_array(vao);
            // gl::draw_arrays(GL_TRIANGLES, 0, 3);
            gl::draw_elements_offset(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);
            // gl::bind_vertex_array(0); // no need to unbind it every time

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
}
