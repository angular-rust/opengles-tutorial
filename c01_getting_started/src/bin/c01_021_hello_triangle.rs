#![allow(dead_code)]
#![allow(unused_variables)]
use dx::gles::{core30::gl, enums::*};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::{ffi::CStr, mem, str, sync::Arc};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct SwapChainDescriptor {}

impl SwapChainDescriptor {
    fn new() -> Self {
        Self {}
    }
}

impl Default for SwapChainDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SwapChain {
    egl: Arc<egl::Instance<egl::Dynamic<libloading::Library, egl::EGL1_4>>>,
    display: egl::Display,
}

impl SwapChain {
    pub fn update(&self, surface: &Surface) {
        self.egl
            .swap_buffers(self.display, surface.raw)
            .expect("unable to post EGL context");
    }
}

pub struct Surface {
    raw: egl::Surface,
}

impl Surface {}

pub struct Queue;

impl Queue {
    fn new() -> Self {
        Self {}
    }

    pub fn submit(&self) {}
}

pub struct CommandEncoder;

impl CommandEncoder {
    pub fn begin_render_pass(&self) {}

    pub fn finish(&self) {}
}

pub struct Device {
    egl: Arc<egl::Instance<egl::Dynamic<libloading::Library, egl::EGL1_4>>>,
    display: egl::Display,
}

impl Device {
    pub fn create_swap_chain(&self, surface: &Surface, desc: &SwapChainDescriptor) -> SwapChain {
        SwapChain {
            egl: self.egl.clone(),
            display: self.display,
        }
    }

    pub fn create_command_encoder(&self) -> CommandEncoder {
        unimplemented!()
    }
}

pub struct Adapter {
    egl: Arc<egl::Instance<egl::Dynamic<libloading::Library, egl::EGL1_4>>>,
    display: egl::Display,
}

impl Adapter {
    pub fn request_device(&self) -> (Device, Queue) {
        let device = Device {
            egl: self.egl.clone(),
            display: self.display,
        };

        let queue = Queue {};

        (device, queue)
    }
}

pub struct Instance {
    egl: Arc<egl::Instance<egl::Dynamic<libloading::Library, egl::EGL1_4>>>,
    display: egl::Display,
}

impl Instance {
    pub fn new() -> Self {
        // let lib = libloading::Library::new("libEGL.so.1").expect("unable to find libEGL.so.1");
        // let egl = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required_from(lib).expect("unable to load libEGL.so.1") };

        // EGL setup here
        let egl = unsafe {
            Arc::new(egl::DynamicInstance::<egl::EGL1_4>::load_required().expect("unable to load libEGL.so.1"))
        };

        // Setup OpenGL ES API
        egl.bind_api(egl::OPENGL_ES_API)
            .expect("unable to select OpenGL ES API"); // for OpenGL ES

        // Setup Display
        let display = egl.get_display(egl::DEFAULT_DISPLAY).expect("unable to get display");

        egl.initialize(display).expect("unable to init EGL");

        unsafe {
            libloading::Library::new("libGLESv2.so.2").expect("unable to find libGLESv2.so.2");
            dx::gles::ffi::load_global_gl_with(|c_str| {
                let procname = CStr::from_ptr(c_str).to_str().unwrap();
                egl.get_proc_address(procname).unwrap() as *mut std::ffi::c_void
            });
            // unsafe { mem::transmute(egli::egl::get_proc_address(s)) }
        };

        Self { egl, display }
    }

    pub fn request_adapter(&self) -> Adapter {
        Adapter {
            egl: self.egl.clone(),
            display: self.display,
        }
    }

    pub fn create_surface(&self, window: &Window) -> Surface {
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

        // // Get the number of matching configurations.
        // let count = egl
        //     .matching_config_count(display, &attrib_list)
        //     .expect("no available configurations");

        // Get the matching configuration.
        let config = self
            .egl
            .choose_first_config(self.display, &attrib_list)
            .expect("unable to choose EGL configuration")
            .expect("no EGL configuration found");

        // if ou not set CONTEXT_CLIENT_VERSION if will set to GLES v.1, otherwise 2 or more;
        let ctx_attribs = [egl::CONTEXT_CLIENT_VERSION, 2, egl::NONE]; // GLESv2/3+
                                                                       // let ctx_attribs = [egl::NONE]; // GLESv1+
        let ctx = self
            .egl
            .create_context(self.display, config, None, &ctx_attribs)
            .expect("unable to create EGL context");

        // Create a EGL surface
        let surface = unsafe {
            let window_handle = match window.raw_window_handle() {
                RawWindowHandle::Xlib(handle) => handle.window as egl::NativeWindowType,
                RawWindowHandle::Xcb(handle) => handle.window as egl::NativeWindowType,
                RawWindowHandle::Wayland(handle) => handle.surface as egl::NativeWindowType,
                _ => {
                    panic!("Other handle type");
                }
            };

            self.egl
                .create_window_surface(self.display, config, window_handle, None)
                .expect("unable to create an EGL surface")
        };

        self.egl
            .make_current(self.display, Some(surface), Some(surface), Some(ctx))
            .expect("unable to bind the context");

        Surface { raw: surface }
    }
}

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

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

// window creation

// SCR_WIDTH, SCR_HEIGHT, "LearnOpenGL"

// gl: load all OpenGL function pointers

// gl::load_with(|symbol| window.get_proc_address(symbol));

pub fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("Failed to create window");

    // initialize and configure

    let instance = Instance::new();
    let surface = instance.create_surface(&window);
    let adapter = instance.request_adapter();
    let (device, queue) = adapter.request_device();
    let desc = Default::default();
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
        let vertices: [f32; 9] = [
            -0.5, -0.5, 0.0, // left
            0.5, -0.5, 0.0, // right
            0.0, 0.5, 0.0, // top
        ];

        let vao = gl::gen_vertex_array();
        let vbo = gl::gen_buffer();
        // bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attributes(s).
        gl::bind_vertex_array(vao);

        gl::bind_buffer(GL_ARRAY_BUFFER, vbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &vertices, GL_STATIC_DRAW);

        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, 3 * mem::size_of::<f32>() as i32, 0);
        gl::enable_vertex_attrib_array(0);

        // note that this is allowed, the call to gl::vertex_attrib_pointer registered vbo as the vertex attribute's bound vertex buffer object so afterwards we can safely unbind
        gl::bind_buffer(GL_ARRAY_BUFFER, 0);

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
            gl::bind_vertex_array(vao); // seeing as we only have a single vao there's no need to bind it every time, but we'll do so to keep things a bit more organized
            gl::draw_arrays(GL_TRIANGLES, 0, 3);
            // gl::bind_vertex_array(0); // no need to unbind it every time

            // swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            swapchain.update(&surface);

            // request redraw again
            window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            // redraw here when something changed
        }
        _ => {}
    });
}
