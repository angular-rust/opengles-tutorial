use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::sync::Arc;
use winit::{
    dpi::{LogicalSize, PhysicalSize, Size},
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use opengles::glesv2 as gl;

pub const VERT_SHADER: &str = r"
    attribute vec3 Position;

    void main()
    {
        gl_Position = vec4(Position, 1.0);
    }
";

pub const FRAG_SHADER: &str = r"
    precision mediump float;
    void main()
    {
        gl_FragColor = vec4(1.0, 0.5, 0.2, 1.0);
    }
";

fn shader_from_source(source: &str, kind: gl::GLenum) -> Result<u32, String> {
    let id = gl::create_shader(kind);

    gl::shader_source(id, source.as_bytes());
    gl::compile_shader(id);

    let success = gl::get_shaderiv(id, gl::GL_COMPILE_STATUS);

    if success == 0 {
        let len = gl::get_shaderiv(id, gl::GL_INFO_LOG_LENGTH);

        return match gl::get_shader_info_log(id, len) {
            Some(message) => Err(message),
            None => Ok(id)
        };
    }

    Ok(id)
}

fn program_from_shaders(vert_shader: u32, frag_shader: u32) -> Result<u32, String> {
    let program_id = gl::create_program();

    gl::attach_shader(program_id, vert_shader);
    gl::attach_shader(program_id, frag_shader);

    gl::link_program(program_id);

    // error handling here
    let success = gl::get_programiv(program_id, gl::GL_LINK_STATUS);

    if success == 0 {
        let len = gl::get_programiv(program_id, gl::GL_INFO_LOG_LENGTH);     

        return match gl::get_program_info_log(program_id, len) {
            Some(message) => Err(message),
            None => Ok(program_id)
        };
    }

    gl::detach_shader(program_id, vert_shader);
    gl::detach_shader(program_id, frag_shader);

    Ok(program_id)
}

fn main() {
    // EGL setup here
    let egl = unsafe {
        Arc::new(
            egl::DynamicInstance::<egl::EGL1_4>::load_required()
                .expect("unable to load libEGL.so.1"),
        )
    };

    // Setup OpenGL ES API
    egl.bind_api(egl::OPENGL_ES_API)
        .expect("unable to select OpenGL ES API"); // for OpenGL ES

    // Setup Display
    let display = egl
        .get_display(egl::DEFAULT_DISPLAY)
        .expect("unable to get display");
    egl.initialize(display).expect("unable to init EGL");

    // Create context
    let attrib_list = [
        egl::BUFFER_SIZE,
        16,
        egl::DEPTH_SIZE,
        16,
        egl::STENCIL_SIZE,
        0,
        egl::SURFACE_TYPE,
        egl::WINDOW_BIT,
        egl::NONE,
    ];

    // Get the matching configuration.
    let config = egl
        .choose_first_config(display, &attrib_list)
        .expect("unable to choose EGL configuration")
        .expect("no EGL configuration found");

    let ctx_attribs = [egl::CONTEXT_CLIENT_VERSION, 2, egl::NONE];
    let ctx = egl
        .create_context(display, config, None, &ctx_attribs)
        .expect("unable to create EGL context");

    // winit stuff
    let event_loop = EventLoop::new();

    let wb = WindowBuilder::new()
        .with_min_inner_size(Size::Logical(LogicalSize::new(64.0, 64.0)))
        .with_inner_size(Size::Physical(PhysicalSize::new(900, 700)))
        .with_title("Game".to_string());

    let window = wb.build(&event_loop).unwrap();

    // Create a EGL surface
    let surface = unsafe {
        let window_handle = match window.raw_window_handle() {
            RawWindowHandle::Xlib(handle) => {
                handle.window as egl::NativeWindowType
            }
            RawWindowHandle::Xcb(handle) => {
                handle.window as egl::NativeWindowType
            }
            RawWindowHandle::Wayland(handle) => {
                handle.surface as egl::NativeWindowType
            }
            _ => {
                panic!("Other handle type");
            }
        };

        egl.create_window_surface(display, config, window_handle, None)
            .expect("unable to create an EGL surface")
    };

    egl.make_current(display, Some(surface), Some(surface), Some(ctx))
        .expect("unable to bind the context");

    println!(
        "GL_RENDERER = {}",
        gl::get_string(gl::GL_RENDERER).unwrap_or("Unknown".into())
    );
    println!(
        "GL_VERSION = {}",
        gl::get_string(gl::GL_VERSION).unwrap_or("Unknown".into())
    );
    println!(
        "GL_VENDOR = {}",
        gl::get_string(gl::GL_VENDOR).unwrap_or("Unknown".into())
    );
    println!(
        "GL_EXTENSIONS = {}",
        gl::get_string(gl::GL_EXTENSIONS).unwrap_or("Unknown".into())
    );

    gl::viewport(0, 0, 900, 700);

    gl::clear_color(0.3, 0.3, 0.5, 1.0);

    gl::clear(gl::GL_COLOR_BUFFER_BIT);


    egl.swap_buffers(display, surface)
        .expect("unable to post EGL context");

    let vertex_shader = match shader_from_source(VERT_SHADER, gl::GL_VERTEX_SHADER) {
        Ok(id) => {
            println!("Vertex Shader Compiled");
            id
        },
        Err(err) => panic!("{}", err)
    };

    let fragment_shader = match shader_from_source(FRAG_SHADER, gl::GL_FRAGMENT_SHADER) {
        Ok(id) => {
            println!("Fragment Shader Compiled");
            id
        },
        Err(err) => panic!("Error: {}", err)
    };

    match program_from_shaders(vertex_shader, fragment_shader) {
        Ok(_) => println!("Program linked"),
        Err(err) => panic!("Error: {}", err)
    }

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
                gl::clear(gl::GL_COLOR_BUFFER_BIT);

                egl.swap_buffers(display, surface)
                    .expect("unable to post EGL context");
            }
            _ => {}
        }
    });
}
