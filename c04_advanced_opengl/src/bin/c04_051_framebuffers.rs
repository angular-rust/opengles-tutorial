use c04_advanced_opengl::{load_texture, process_events, process_input, Camera, Shader};
use cgmath::prelude::*;
use cgmath::{perspective, vec3, Deg, Matrix4, Point3};
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

    // query framebuffer size as it might be quite different from the requested size on Retina displays
    // let (scr_width, scr_height) = window.get_framebuffer_size();
    let (scr_width, scr_height) = (SCR_WIDTH as i32, SCR_HEIGHT as i32);

    // tell to capture our mouse
    // window.set_cursor_mode(CursorMode::Disabled);

    let (
        shader,
        screen_shader,
        cubevao,
        planevao,
        quadvao,
        cube_texture,
        floor_texture,
        framebuffer,
        texture_colorbuffer,
    ) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);

        // build and compile our shader program

        let shader = Shader::new(assets!("shaders/5.1.framebuffers.vs"), assets!("shaders/5.1.framebuffers.fs"));
        let screen_shader =
            Shader::new(assets!("shaders/5.1.framebuffers_screen.vs"), assets!("shaders/5.1.framebuffers_screen.fs"));

        // set up vertex data (and buffer(s)) and configure vertex attributes

        let cube_vertices: [f32; 180] = [
            // positions       // texture Coords
            -0.5, -0.5, -0.5, 0.0, 0.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.5, 0.5, -0.5, 1.0, 1.0, 0.5, 0.5, -0.5, 1.0, 1.0,
            -0.5, 0.5, -0.5, 0.0, 1.0, -0.5, -0.5, -0.5, 0.0, 0.0, -0.5, -0.5, 0.5, 0.0, 0.0, 0.5, -0.5, 0.5, 1.0, 0.0,
            0.5, 0.5, 0.5, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 1.0, -0.5, 0.5, 0.5, 0.0, 1.0, -0.5, -0.5, 0.5, 0.0, 0.0,
            -0.5, 0.5, 0.5, 1.0, 0.0, -0.5, 0.5, -0.5, 1.0, 1.0, -0.5, -0.5, -0.5, 0.0, 1.0, -0.5, -0.5, -0.5, 0.0,
            1.0, -0.5, -0.5, 0.5, 0.0, 0.0, -0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, -0.5, 1.0,
            1.0, 0.5, -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, 0.5, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0,
            0.0, -0.5, -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, -0.5, 1.0, 1.0, 0.5, -0.5, 0.5, 1.0, 0.0, 0.5, -0.5, 0.5, 1.0,
            0.0, -0.5, -0.5, 0.5, 0.0, 0.0, -0.5, -0.5, -0.5, 0.0, 1.0, -0.5, 0.5, -0.5, 0.0, 1.0, 0.5, 0.5, -0.5, 1.0,
            1.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, -0.5, 0.5, 0.5, 0.0, 0.0, -0.5, 0.5, -0.5, 0.0, 1.0,
        ];
        let plane_vertices: [f32; 30] = [
            // positions       // texture Coords (note we set these higher than 1 (together with GL_REPEAT as texture wrapping mode). this will cause the floor texture to repeat)
            5.0, -0.5, 5.0, 2.0, 0.0, -5.0, -0.5, 5.0, 0.0, 0.0, -5.0, -0.5, -5.0, 0.0, 2.0, 5.0, -0.5, 5.0, 2.0, 0.0,
            -5.0, -0.5, -5.0, 0.0, 2.0, 5.0, -0.5, -5.0, 2.0, 2.0,
        ];
        let quad_vertices: [f32; 24] = [
            // vertex attributes for a quad that fills the entire screen in Normalized Device Coordinates.
            // positions // texCoords
            -1.0, 1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 0.0,
            1.0, 1.0, 1.0, 1.0,
        ];

        // cube vao
        let cubevao = gl::gen_vertex_array();
        let cubevbo = gl::gen_buffer();
        gl::bind_vertex_array(cubevao);
        gl::bind_buffer(GL_ARRAY_BUFFER, cubevbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &cube_vertices, GL_STATIC_DRAW);
        let stride = 5 * mem::size_of::<f32>() as i32;
        gl::enable_vertex_attrib_array(0);
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(1);
        gl::vertex_attrib_pointer_offset(1, 2, GL_FLOAT, false, stride, 3 * mem::size_of::<f32>() as u32);
        gl::bind_vertex_array(0);
        // plane vao

        let planevao = gl::gen_vertex_array();
        let planevbo = gl::gen_buffer();
        gl::bind_vertex_array(planevao);
        gl::bind_buffer(GL_ARRAY_BUFFER, planevbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &plane_vertices, GL_STATIC_DRAW);
        gl::enable_vertex_attrib_array(0);
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(1);
        gl::vertex_attrib_pointer_offset(1, 2, GL_FLOAT, false, stride, 3 * mem::size_of::<f32>() as u32);

        // screen quad vao
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

        // load textures

        let cube_texture = load_texture(assets!("textures/container.jpg"));
        let floor_texture = load_texture(assets!("textures/metal.png"));

        // shader configuration

        shader.use_program();
        shader.set_int("texture1", 0);

        screen_shader.use_program();
        screen_shader.set_int("screenTexture", 0);

        // framebuffer configuration

        let framebuffer = gl::gen_framebuffer();
        gl::bind_framebuffer(GL_FRAMEBUFFER, framebuffer);

        // create a color attachment texture
        let texture_colorbuffer = gl::gen_texture();
        gl::bind_texture(GL_TEXTURE_2D, texture_colorbuffer);
        gl::empty_tex_image_2d(GL_TEXTURE_2D, 0, GL_RGB as i32, scr_width, scr_height, 0, GL_RGB, GL_UNSIGNED_BYTE);
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
        gl::framebuffer_texture_2d(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, texture_colorbuffer, 0);
        // create a renderbuffer object for depth and stencil attachment (we won't be sampling these)
        let rbo = gl::gen_renderbuffer();
        gl::bind_renderbuffer(GL_RENDERBUFFER, rbo);
        // use a single renderbuffer object for both a depth AND stencil buffer.
        gl::renderbuffer_storage(GL_RENDERBUFFER, GL_DEPTH24_STENCIL8, scr_width, scr_height);
        // now actually attach it
        gl::framebuffer_renderbuffer(GL_FRAMEBUFFER, GL_DEPTH_STENCIL_ATTACHMENT, GL_RENDERBUFFER, rbo);
        // now that we actually created the framebuffer and added all attachments we want to check if it is actually complete now
        if gl::check_framebuffer_status(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
        }
        gl::bind_framebuffer(GL_FRAMEBUFFER, 0);

        // draw as wireframe
        // gl::PolygonMode(GL_FRONT_AND_BACK, GL_LINE);

        (
            shader,
            screen_shader,
            cubevao,
            planevao,
            quadvao,
            cube_texture,
            floor_texture,
            framebuffer,
            texture_colorbuffer,
        )
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

            // bind to framebuffer and draw scene as we normally would to color texture
            gl::bind_framebuffer(GL_FRAMEBUFFER, framebuffer);
            gl::enable(GL_DEPTH_TEST); // enable depth testing (is disabled for rendering screen-space quad)

            // make sure we clear the framebuffer's content
            gl::clear_color(0.1, 0.1, 0.1, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            shader.use_program();
            let mut model: Matrix4<f32>;
            let view = camera.get_view_matrix();
            let projection: Matrix4<f32> =
                perspective(Deg(camera.zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            shader.set_mat4("view", &view);
            shader.set_mat4("projection", &projection);
            // cubes
            gl::bind_vertex_array(cubevao);
            gl::active_texture(GL_TEXTURE0);
            gl::bind_texture(GL_TEXTURE_2D, cube_texture);
            model = Matrix4::from_translation(vec3(-1.0, 0.0, -1.0));
            shader.set_mat4("model", &model);
            gl::draw_arrays(GL_TRIANGLES, 0, 36);
            model = Matrix4::from_translation(vec3(2.0, 0.0, 0.0));
            shader.set_mat4("model", &model);
            gl::draw_arrays(GL_TRIANGLES, 0, 36);
            // floor
            gl::bind_vertex_array(planevao);
            gl::bind_texture(GL_TEXTURE_2D, floor_texture);
            shader.set_mat4("model", &Matrix4::identity());
            gl::draw_arrays(GL_TRIANGLES, 0, 6);
            gl::bind_vertex_array(0);

            // now bind back to default framebuffer and draw a quad plane with the attached framebuffer color texture
            gl::bind_framebuffer(GL_FRAMEBUFFER, 0);
            // disable depth test so screen-space quad isn't discarded due to depth test.
            gl::disable(GL_DEPTH_TEST);
            // clear all relevant buffers
            // set clear color to white (not really necessery actually, since we won't be able to see behind the quad anyways)
            gl::clear_color(1.0, 0.5, 1.0, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT);

            screen_shader.use_program();
            gl::bind_vertex_array(quadvao);
            // use the color attachment texture as the texture of the quad plane
            gl::bind_texture(GL_TEXTURE_2D, texture_colorbuffer);
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

    // optional: de-allocate all resources once they've outlived their purpose:
    // gl::delete_vertex_arrays(1, &cubevao);
    // gl::delete_vertex_arrays(1, &planevao);
    // gl::delete_vertex_arrays(1, &quadvao);
    // gl::delete_buffers(1, &cubevbo);
    // gl::delete_buffers(1, &planevbo);
    // gl::delete_buffers(1, &quadvbo);
}
