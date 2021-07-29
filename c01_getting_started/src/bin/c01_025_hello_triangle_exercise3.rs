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

const FRAGMENT_SHADER2_SOURCE: &str = r#"
    #version 330 core
    out vec4 FragColor;
    void main() {
       FragColor = vec4(1.0f, 1.0f, 0.0f, 1.0f);
    }
"#;

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

    let (shader_program_orange, shader_program_yellow, vaos) = {
        // build and compile our shader program

        // // we skipped compile log checks this time for readability (if you do encounter issues, add the compile-checks! see previous code samples)
        let vertex_shader = gl::create_shader(GL_VERTEX_SHADER);
        let fragment_shader_orange = gl::create_shader(GL_FRAGMENT_SHADER); // the first fragment shader that outputs the color orange
        let fragment_shader_yellow = gl::create_shader(GL_FRAGMENT_SHADER); // the second fragment shader that outputs the color yellow
        let shader_program_orange = gl::create_program();
        let shader_program_yellow = gl::create_program(); // the second shader program

        gl::shader_source(vertex_shader, VERTEX_SHADER_SOURCE.as_bytes());
        gl::compile_shader(vertex_shader);

        gl::shader_source(fragment_shader_orange, FRAGMENT_SHADER_SOURCE.as_bytes());
        gl::compile_shader(fragment_shader_orange);

        gl::shader_source(fragment_shader_yellow, FRAGMENT_SHADER2_SOURCE.as_bytes());
        gl::compile_shader(fragment_shader_yellow);
        // link the first program object
        gl::attach_shader(shader_program_orange, vertex_shader);
        gl::attach_shader(shader_program_orange, fragment_shader_orange);
        gl::link_program(shader_program_orange);
        // then link the second program object using a different fragment shader (but same vertex shader)
        // this is perfectly allowed since the inputs and outputs of both the vertex and fragment shaders are equally matched.
        gl::attach_shader(shader_program_yellow, vertex_shader);
        gl::attach_shader(shader_program_yellow, fragment_shader_yellow);
        gl::link_program(shader_program_yellow);

        // set up vertex data (and buffer(s)) and configure vertex attributes

        let first_triangle: [f32; 9] = [
            -0.9, -0.5, 0.0, // left
            -0.0, -0.5, 0.0, // right
            -0.45, 0.5, 0.0, // top
        ];
        let second_triangle: [f32; 9] = [
            0.0, -0.5, 0.0, // left
            0.9, -0.5, 0.0, // right
            0.45, 0.5, 0.0, // top
        ];

        let vaos = {
            let v = gl::gen_vertex_arrays(2); // we can also generate multiple vaos or buffers at the same time
            [v[0], v[1]]
        };

        let vbos = {
            let v = gl::gen_buffers(2);
            [v[0], v[1]]
        };
        // first triangle setup

        gl::bind_vertex_array(vaos[0]);
        gl::bind_buffer(GL_ARRAY_BUFFER, vbos[0]);
        // Vertex attributes stay the same
        gl::buffer_data(GL_ARRAY_BUFFER, &first_triangle, GL_STATIC_DRAW);

        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, 3 * mem::size_of::<f32>() as i32, 0);
        gl::enable_vertex_attrib_array(0);
        // gl::bind_vertex_array(0); // no need to unbind at all as we directly bind a different vao the next few lines
        // second triangle setup

        gl::bind_vertex_array(vaos[1]);
        gl::bind_buffer(GL_ARRAY_BUFFER, vbos[1]);
        gl::buffer_data(GL_ARRAY_BUFFER, &second_triangle, GL_STATIC_DRAW);

        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, 0, 0); // because the vertex data is tightly packed we can also specify 0 as the vertex attribute's stride to let OpenGL figure it out
        gl::enable_vertex_attrib_array(0);
        // gl::bind_vertex_array(0); // not really necessary as well, but beware of calls that could affect vaos while this one is bound (like binding element buffer objects, or enabling/disabling vertex attributes)

        // uncomment this call to draw in wireframe polygons.
        // gl::PolygonMode(GL_FRONT_AND_BACK, GL_LINE);

        (shader_program_orange, shader_program_yellow, vaos)
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

            // now when we draw the triangle we first use the vertex and orange fragment shader from the first program
            gl::use_program(shader_program_orange);
            // draw the first triangle using the data from our first vao
            gl::bind_vertex_array(vaos[0]);
            gl::draw_arrays(GL_TRIANGLES, 0, 3); // this call should output an orange triangle
                                                 // then we draw the second triangle using the data from the second vao
                                                 // when we draw the second triangle we want to use a different shader program so we switch to the shader program with our yellow fragment shader.
            gl::use_program(shader_program_yellow);
            gl::bind_vertex_array(vaos[1]);
            gl::draw_arrays(GL_TRIANGLES, 0, 3);

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

    // // optional: de-allocate all resources once they've outlived their purpose:
    //     gl::delete_vertex_arrays(2, vaos.as_mut_ptr());
    //     gl::delete_buffers(2, vbos.as_mut_ptr());
}
