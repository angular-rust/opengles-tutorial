#![allow(dead_code)]
#![allow(unused_variables)]
use c07_in_practice::Shader;
use cgmath::prelude::*;
use cgmath::{perspective, vec3, Deg, Matrix4, Rad};
use dx::{
    assets,
    gles::{core30::gl, enums::*, GLchar, GLenum, GLsizei, GLuint},
    utils::*,
    Instance,
};
use image::GenericImageView;
use std::{ffi::CStr, mem, os::raw::c_void, time::SystemTime};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

#[allow(non_snake_case)]
fn glCheckError_(file: &str, line: u32) -> u32 {
    let mut errorCode = gl::get_error();
    while errorCode != GL_NO_ERROR {
        let error = match errorCode {
            GL_INVALID_ENUM => "INVALID_ENUM",
            GL_INVALID_VALUE => "INVALID_VALUE",
            GL_INVALID_OPERATION => "INVALID_OPERATION",
            GL_STACK_OVERFLOW => "STACK_OVERFLOW",
            GL_STACK_UNDERFLOW => "STACK_UNDERFLOW",
            GL_OUT_OF_MEMORY => "OUT_OF_MEMORY",
            GL_INVALID_FRAMEBUFFER_OPERATION => "INVALID_FRAMEBUFFER_OPERATION",
            _ => "unknown GL error code",
        };

        println!("{} | {} ({})", error, file, line);

        errorCode = gl::get_error();
    }
    errorCode
}

macro_rules! glCheckError {
    () => {
        glCheckError_(file!(), line!())
    };
}

#[allow(non_snake_case)]
extern "system" fn glDebugOutput(
    source: GLenum,
    type_: GLenum,
    id: GLuint,
    severity: GLenum,
    _length: GLsizei,
    message: *const GLchar,
    _userParam: *mut c_void,
) {
    if id == 131_169 || id == 131_185 || id == 131_218 || id == 131_204 {
        // ignore these non-significant error codes
        return;
    }

    println!("--");
    let message = unsafe { CStr::from_ptr(message).to_str().unwrap() };
    println!("Debug message ({}): {}", id, message);
    match source {
        GL_DEBUG_SOURCE_API => println!("Source: API"),
        GL_DEBUG_SOURCE_WINDOW_SYSTEM => println!("Source: Window System"),
        GL_DEBUG_SOURCE_SHADER_COMPILER => println!("Source: Shader Compiler"),
        GL_DEBUG_SOURCE_THIRD_PARTY => println!("Source: Third Party"),
        GL_DEBUG_SOURCE_APPLICATION => println!("Source: Application"),
        GL_DEBUG_SOURCE_OTHER => println!("Source: Other"),
        _ => println!("Source: Unknown enum value"),
    }

    match type_ {
        GL_DEBUG_TYPE_ERROR => println!("Type: Error"),
        GL_DEBUG_TYPE_DEPRECATED_BEHAVIOR => println!("Type: Deprecated Behaviour"),
        GL_DEBUG_TYPE_UNDEFINED_BEHAVIOR => println!("Type: Undefined Behaviour"),
        GL_DEBUG_TYPE_PORTABILITY => println!("Type: Portability"),
        GL_DEBUG_TYPE_PERFORMANCE => println!("Type: Performance"),
        GL_DEBUG_TYPE_MARKER => println!("Type: Marker"),
        GL_DEBUG_TYPE_PUSH_GROUP => println!("Type: Push Group"),
        GL_DEBUG_TYPE_POP_GROUP => println!("Type: Pop Group"),
        GL_DEBUG_TYPE_OTHER => println!("Type: Other"),
        _ => println!("Type: Unknown enum value"),
    }

    match severity {
        GL_DEBUG_SEVERITY_HIGH => println!("Severity: high"),
        GL_DEBUG_SEVERITY_MEDIUM => println!("Severity: medium"),
        GL_DEBUG_SEVERITY_LOW => println!("Severity: low"),
        GL_DEBUG_SEVERITY_NOTIFICATION => println!("Severity: notification"),
        _ => println!("Severity: Unknown enum value"),
    }
}

pub fn main() {
    let start_time = SystemTime::now();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // initialize and configure
    let instance = Instance::new();
    let surface = instance.create_surface(&window);
    let adapter = instance.request_adapter();
    let (device, queue) = adapter.request_device();
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

    let (shader, cubevao, texture) = {
        // enable OpenGL debug context if context allows for debug context
        let flags = gl::get_integerv(GL_CONTEXT_FLAGS);
        if flags as u32 & GL_CONTEXT_FLAG_DEBUG_BIT != 0 {
            gl::enable(GL_DEBUG_OUTPUT);
            gl::enable(GL_DEBUG_OUTPUT_SYNCHRONOUS); // makes sure errors are displayed synchronously
                                                     // TODO: core 3.2
                                                     // gl::debug_message_callback(glDebugOutput, ptr::null());
                                                     // gl::debug_message_control(GL_DONT_CARE, GL_DONT_CARE, GL_DONT_CARE, 0, ptr::null(), GL_TRUE);
        } else {
            println!("Debug Context not active! Check if your driver supports the extension.")
        }

        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);
        gl::enable(GL_CULL_FACE);

        // OpenGL initial state
        let shader = Shader::new(
            assets!("shaders/debugging.vs").to_str().unwrap(),
            assets!("shaders/debugging.fs").to_str().unwrap(),
        );

        // configure 3D cube
        let vertices: [f32; 180] = [
            // back face
            -0.5, -0.5, -0.5, 0.0, 0.0, // Bottom-left
            0.5, 0.5, -0.5, 1.0, 1.0, // top-right
            0.5, -0.5, -0.5, 1.0, 0.0, // bottom-right
            0.5, 0.5, -0.5, 1.0, 1.0, // top-right
            -0.5, -0.5, -0.5, 0.0, 0.0, // bottom-left
            -0.5, 0.5, -0.5, 0.0, 1.0, // top-left
            // front face
            -0.5, -0.5, 0.5, 0.0, 0.0, // bottom-left
            0.5, -0.5, 0.5, 1.0, 0.0, // bottom-right
            0.5, 0.5, 0.5, 1.0, 1.0, // top-right
            0.5, 0.5, 0.5, 1.0, 1.0, // top-right
            -0.5, 0.5, 0.5, 0.0, 1.0, // top-left
            -0.5, -0.5, 0.5, 0.0, 0.0, // bottom-left
            // left face
            -0.5, 0.5, 0.5, -1.0, 0.0, // top-right
            -0.5, 0.5, -0.5, -1.0, 1.0, // top-left
            -0.5, -0.5, -0.5, -0.0, 1.0, // bottom-left
            -0.5, -0.5, -0.5, -0.0, 1.0, // bottom-left
            -0.5, -0.5, 0.5, -0.0, 0.0, // bottom-right
            -0.5, 0.5, 0.5, -1.0, 0.0, // top-right
            // right face
            0.5, 0.5, 0.5, 1.0, 0.0, // top-left
            0.5, -0.5, -0.5, 0.0, 1.0, // bottom-right
            0.5, 0.5, -0.5, 1.0, 1.0, // top-right
            0.5, -0.5, -0.5, 0.0, 1.0, // bottom-right
            0.5, 0.5, 0.5, 1.0, 0.0, // top-left
            0.5, -0.5, 0.5, 0.0, 0.0, // bottom-left
            // bottom face
            -0.5, -0.5, -0.5, 0.0, 1.0, // top-right
            0.5, -0.5, -0.5, 1.0, 1.0, // top-left
            0.5, -0.5, 0.5, 1.0, 0.0, // bottom-left
            0.5, -0.5, 0.5, 1.0, 0.0, // bottom-left
            -0.5, -0.5, 0.5, 0.0, 0.0, // bottom-right
            -0.5, -0.5, -0.5, 0.0, 1.0, // top-right
            // top face
            -0.5, 0.5, -0.5, 0.0, 1.0, // top-left
            0.5, 0.5, 0.5, 1.0, 0.0, // bottom-right
            0.5, 0.5, -0.5, 1.0, 1.0, // top-right
            0.5, 0.5, 0.5, 1.0, 0.0, // bottom-right
            -0.5, 0.5, -0.5, 0.0, 1.0, // top-left
            -0.5, 0.5, 0.5, 0.0, 0.0, // bottom-left
        ];

        let cubevao = gl::gen_vertex_array();
        let cubevbo = gl::gen_buffer();
        // fill buffer
        gl::bind_buffer(GL_ARRAY_BUFFER, cubevbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &vertices, GL_STATIC_DRAW);
        // link vertex attributes
        gl::bind_vertex_array(cubevao);
        gl::enable_vertex_attrib_array(0);
        let stride = 5 * mem::size_of::<f32>() as i32;
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(1);
        gl::vertex_attrib_pointer_offset(1, 2, GL_FLOAT, false, stride, 3 * mem::size_of::<f32>() as u32);
        gl::bind_buffer(GL_ARRAY_BUFFER, 0);
        gl::bind_vertex_array(0);

        // load cube texture
        let texture = gl::gen_texture();
        gl::bind_texture(GL_TEXTURE_2D, texture);
        let img = image::open(assets!("textures/wood.png")).expect("Failed to load texture");
        let data = img.as_bytes();
        gl::tex_image_2d(
            GL_TEXTURE_2D,
            0,
            GL_RGB as i32,
            img.width() as i32,
            img.height() as i32,
            0,
            GL_RGB,
            GL_UNSIGNED_BYTE,
            &data,
        );
        gl::generate_mipmap(GL_TEXTURE_2D);

        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT as i32);
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT as i32);
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR as i32);
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);

        // set up projection matrix
        let projection: Matrix4<f32> = perspective(Deg(45.0), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
        shader.set_mat4("projection", &projection);
        shader.set_int("tex", 0);

        glCheckError!();

        (shader, cubevao, texture)
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
            // Set the viewport

            // render

            gl::clear_color(0.2, 0.3, 0.3, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            shader.use_program();
            let rotation_speed = 10.0;
            let angle = get_time(&start_time) * rotation_speed;
            let mut model: Matrix4<f32> = Matrix4::from_translation(vec3(0., 0., -2.5));
            model = model * Matrix4::from_axis_angle(vec3(1.0, 1.0, 1.0).normalize(), Rad(angle));
            shader.set_mat4("model", &model);

            gl::bind_texture(GL_TEXTURE_2D, texture);
            gl::bind_vertex_array(cubevao);
            gl::draw_arrays(GL_TRIANGLES, 0, 36);
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
}
