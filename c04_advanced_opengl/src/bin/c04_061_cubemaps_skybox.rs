use c04_advanced_opengl::{load_texture, process_events, process_input, Camera, Shader};
use cgmath::prelude::*;
use cgmath::{perspective, Deg, Matrix4, Point3};
use dx::{
    assets,
    gles::{core30::gl, enums::*},
    utils::*,
    Instance,
};
use image::GenericImageView;
use std::path::PathBuf;
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

    let (shader, skybox_shader, cubevao, skyboxvao, cube_texture, cubemap_texture) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);
        gl::depth_func(GL_ALWAYS); // always pass the depth test (same effect as glDisable(GL_DEPTH_TEST))

        // build and compile our shader program
        let shader = Shader::new(assets!("shaders/6.1.cubemaps.vs"), assets!("shaders/6.1.cubemaps.fs"));
        let skybox_shader = Shader::new(assets!("shaders/6.1.skybox.vs"), assets!("shaders/6.1.skybox.fs"));

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
        let skybox_vertices: [f32; 108] = [
            // positions
            -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0,
            -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0,
            -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0,
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0,
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0,
            -1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
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

        // skybox vao
        let skyboxvao = gl::gen_vertex_array();
        let skyboxvbo = gl::gen_buffer();
        gl::bind_vertex_array(skyboxvao);
        gl::bind_buffer(GL_ARRAY_BUFFER, skyboxvbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &skybox_vertices, GL_STATIC_DRAW);
        gl::enable_vertex_attrib_array(0);
        let stride = 3 * mem::size_of::<f32>() as i32;
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);

        // load textures

        let cube_texture = load_texture(assets!("textures/container.jpg"));

        let faces = [
            assets!("textures/skybox/right.jpg"),
            assets!("textures/skybox/left.jpg"),
            assets!("textures/skybox/top.jpg"),
            assets!("textures/skybox/bottom.jpg"),
            assets!("textures/skybox/back.jpg"),
            assets!("textures/skybox/front.jpg"),
        ];
        let cubemap_texture = load_cubemap(&faces);

        // shader configuration

        shader.use_program();
        shader.set_int("texture1", 0);

        skybox_shader.use_program();
        skybox_shader.set_int("skybox", 0);

        (shader, skybox_shader, cubevao, skyboxvao, cube_texture, cubemap_texture)
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
            let model: Matrix4<f32> = Matrix4::identity();
            let mut view = camera.get_view_matrix();
            let projection: Matrix4<f32> =
                perspective(Deg(camera.zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            shader.set_mat4("model", &model);
            shader.set_mat4("view", &view);
            shader.set_mat4("projection", &projection);
            // cubes
            gl::bind_vertex_array(cubevao);
            gl::active_texture(GL_TEXTURE0);
            gl::bind_texture(GL_TEXTURE_2D, cube_texture);
            gl::draw_arrays(GL_TRIANGLES, 0, 36);

            // draw skybox as last
            gl::depth_func(GL_LEQUAL); // change depth function so depth test passes when values are equal to depth buffer's content
            skybox_shader.use_program();
            // remove translation from the view matrix
            view = camera.get_view_matrix();
            view.w[0] = 0.0;
            view.w[1] = 0.0;
            view.w[2] = 0.0;
            skybox_shader.set_mat4("view", &view);
            skybox_shader.set_mat4("projection", &projection);
            // skybox cube
            gl::bind_vertex_array(skyboxvao);
            gl::active_texture(GL_TEXTURE0);
            gl::bind_texture(GL_TEXTURE_CUBE_MAP, cubemap_texture);
            gl::draw_arrays(GL_TRIANGLES, 0, 36);
            gl::bind_vertex_array(0);
            gl::depth_func(GL_LESS); // set depth function back to default

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
    // gl::delete_vertex_arrays(1, &skyboxvao);
    // gl::delete_buffers(1, &cubevbo);
    // gl::delete_buffers(1, &skyboxvbo);
}

/// loads a cubemap texture from 6 individual texture faces
/// order:
/// +X (right)
/// -X (left)
/// +Y (top)
/// -Y (bottom)
/// +Z (front)
/// -Z (back)

fn load_cubemap(faces: &[PathBuf]) -> u32 {
    let texture_id = gl::gen_texture();
    gl::bind_texture(GL_TEXTURE_CUBE_MAP, texture_id);

    for (i, face) in faces.iter().enumerate() {
        let path = face.as_path();

        let img = image::open(&path).expect("Cubemap texture failed to load");

        let data = img.as_bytes();
        gl::tex_image_2d(
            GL_TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
            0,
            GL_RGB as i32,
            img.width() as i32,
            img.height() as i32,
            0,
            GL_RGB,
            GL_UNSIGNED_BYTE,
            &data,
        );
    }

    gl::tex_parameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
    gl::tex_parameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
    gl::tex_parameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE as i32);
    gl::tex_parameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE as i32);
    gl::tex_parameteri(GL_TEXTURE_CUBE_MAP, GL_TEXTURE_WRAP_R, GL_CLAMP_TO_EDGE as i32);

    texture_id
}
