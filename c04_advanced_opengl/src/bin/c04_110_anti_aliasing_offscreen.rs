use c04_advanced_opengl::{process_events, process_input, Camera, Shader};
use cgmath::prelude::*;
use cgmath::{perspective, Deg, Matrix4, Point3};
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

    // glfw window creation

    // let (mut window, events) = create_window(SCR_WIDTH, SCR_HEIGHT, "LearnOpenGL", glfw::WindowMode::Windowed)
    //     .expect("Failed to create GLFW window");

    // query framebuffer size as it might be quite different from the requested size on Retina displays
    // let (scr_width, scr_height) = window.get_framebuffer_size();
    let (scr_width, scr_height) = (SCR_WIDTH as i32, SCR_HEIGHT as i32);

    // window.make_current();
    // window.set_framebuffer_size_polling(true);
    // window.set_cursor_pos_polling(true);
    // window.set_scroll_polling(true);

    // tell to capture our mouse
    // window.set_cursor_mode(CursorMode::Disabled);

    let (shader, screen_shader, cubevao, quadvao, framebuffer, intermediate_fbo, screen_texture) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);

        // build and compile our shader program

        let shader = Shader::new(assets!("shaders/11.anti_aliasing.vs"), assets!("shaders/11.anti_aliasing.fs"));
        let screen_shader = Shader::new(assets!("shaders/11.aa_post.vs"), assets!("shaders/11.aa_post.fs"));

        // set up vertex data (and buffer(s)) and configure vertex attributes

        let cube_vertices: [f32; 108] = [
            // positions
            -0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, -0.5,
            -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5,
            -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
            0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5,
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5,
        ];
        let quad_vertices: [f32; 24] = [
            // vertex attributes for a quad that fills the entire screen in Normalized Device Coordinates.
            // positions // texCoords
            -1.0, 1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 0.0,
            1.0, 1.0, 1.0, 1.0,
        ];

        // setup cube vao
        let cubevao = gl::gen_vertex_array();
        let cubevbo = gl::gen_buffer();
        gl::bind_vertex_array(cubevao);
        gl::bind_buffer(GL_ARRAY_BUFFER, cubevbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &cube_vertices, GL_STATIC_DRAW);
        let stride = 3 * mem::size_of::<f32>() as i32;
        gl::enable_vertex_attrib_array(0);
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);

        // setup screen vao
        let quadvao = gl::gen_vertex_array();
        let quadvbo = gl::gen_buffer();
        gl::bind_vertex_array(quadvbo);
        gl::bind_buffer(GL_ARRAY_BUFFER, quadvbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &quad_vertices, GL_STATIC_DRAW);
        gl::enable_vertex_attrib_array(0);
        let stride = 4 * mem::size_of::<f32>() as i32;
        gl::vertex_attrib_pointer_offset(0, 2, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(1);
        gl::vertex_attrib_pointer_offset(1, 2, GL_FLOAT, false, stride, 2 * mem::size_of::<f32>() as u32);

        // configure MSAA framebuffer

        let framebuffer = gl::gen_framebuffer();
        gl::bind_framebuffer(GL_FRAMEBUFFER, framebuffer);
        // create a multisampled color attachment texture
        let texture_color_buffer_multisampled = gl::gen_texture();
        gl::bind_texture(GL_TEXTURE_2D_MULTISAMPLE, texture_color_buffer_multisampled);

        // core 3.1
        // gl::TexImage2dMultisample(GL_TEXTURE_2D_MULTISAMPLE, 4, GL_RGB, scr_width, scr_height, GL_TRUE); //DV
        gl::bind_texture(GL_TEXTURE_2D_MULTISAMPLE, 0);
        gl::framebuffer_texture_2d(
            GL_FRAMEBUFFER,
            GL_COLOR_ATTACHMENT0,
            GL_TEXTURE_2D_MULTISAMPLE,
            texture_color_buffer_multisampled,
            0,
        );
        // create a (also multisampled) renderbuffer object for depth and stencil attachments
        let rbo = gl::gen_renderbuffer();
        gl::bind_renderbuffer(GL_RENDERBUFFER, rbo);
        gl::renderbuffer_storage_multisample(GL_RENDERBUFFER, 4, GL_DEPTH24_STENCIL8, scr_width, scr_height);
        gl::bind_renderbuffer(GL_RENDERBUFFER, 0);

        if gl::check_framebuffer_status(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
        }
        gl::bind_framebuffer(GL_FRAMEBUFFER, 0);

        // configure second post-processing framebuffer
        let intermediate_fbo = gl::gen_framebuffer();
        gl::bind_framebuffer(GL_FRAMEBUFFER, intermediate_fbo);

        // create a color attachment texture
        let screen_texture = gl::gen_texture();
        gl::bind_texture(GL_TEXTURE_2D, screen_texture);
        gl::empty_tex_image_2d(GL_TEXTURE_2D, 0, GL_RGB as i32, scr_width, scr_height, 0, GL_RGB, GL_UNSIGNED_BYTE);
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
        gl::framebuffer_texture_2d(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, screen_texture, 0); // we only need a color buffer

        if gl::check_framebuffer_status(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Intermediate framebuffer is not complete!");
        }
        gl::bind_framebuffer(GL_FRAMEBUFFER, 0);

        screen_shader.use_program();
        screen_shader.set_int("screenTexture", 0);

        (shader, screen_shader, cubevao, quadvao, framebuffer, intermediate_fbo, screen_texture)
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

            // render loop

            gl::clear_color(0.1, 0.1, 0.1, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            // 1. draw scene as normal in multisampled buffers
            gl::bind_framebuffer(GL_FRAMEBUFFER, framebuffer);
            gl::clear_color(0.1, 0.1, 0.1, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            // set transformation matrices
            shader.use_program();
            let projection: Matrix4<f32> =
                perspective(Deg(camera.zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            shader.set_mat4("projection", &projection);
            shader.set_mat4("view", &camera.get_view_matrix());
            shader.set_mat4("model", &Matrix4::identity());

            gl::bind_vertex_array(cubevao);
            gl::draw_arrays(GL_TRIANGLES, 0, 36);

            // 2. now blit multisampled buffer(s) to normal colorbuffer of intermediate FBO. Image is stored in screenTexture
            gl::bind_framebuffer(GL_READ_FRAMEBUFFER, framebuffer);
            gl::bind_framebuffer(GL_DRAW_FRAMEBUFFER, intermediate_fbo);
            gl::blit_framebuffer(
                0,
                0,
                scr_width,
                scr_height,
                0,
                0,
                scr_width,
                scr_height,
                GL_COLOR_BUFFER_BIT,
                GL_NEAREST,
            );

            // 3. now render quad with scene's visuals as its texture image
            gl::bind_framebuffer(GL_FRAMEBUFFER, 0);
            gl::clear_color(1.0, 1.0, 1.0, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT);
            gl::disable(GL_DEPTH_TEST);

            // draw Screen quad
            screen_shader.use_program();
            gl::bind_vertex_array(quadvao);
            gl::active_texture(GL_TEXTURE0);
            gl::bind_texture(GL_TEXTURE_2D, screen_texture); // use the now resolved color attachment as the quad's texture
            gl::draw_arrays(GL_TRIANGLES, 0, 6);

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
