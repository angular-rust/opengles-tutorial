use c04_advanced_opengl::{process_events, process_input, Camera, Shader};
use cgmath::prelude::*;
use cgmath::{perspective, vec3, Deg, Matrix4, Point3};
use dx::{
    assets,
    gles::{core30::gl, enums::*},
    utils::*,
    Instance,
};
use image::{DynamicImage::*, GenericImageView};
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

    let (shader, cubevao, planevao, transparentvao, cube_texture, floor_texture, transparent_texture, mut windows) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);
        gl::enable(GL_BLEND);
        gl::blend_func(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

        // build and compile our shader program
        let shader = Shader::new(assets!("shaders/3.2.blending.vs"), assets!("shaders/3.2.blending.fs"));

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
        let transparent_vertices: [f32; 30] = [
            // positions      // texture Coords (swapped y coordinates because texture is flipped upside down)
            0.0, 0.5, 0.0, 0.0, 0.0, 0.0, -0.5, 0.0, 0.0, 1.0, 1.0, -0.5, 0.0, 1.0, 1.0, 0.0, 0.5, 0.0, 0.0, 0.0, 1.0,
            -0.5, 0.0, 1.0, 1.0, 1.0, 0.5, 0.0, 1.0, 0.0,
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

        // transparent vao
        let transparentvao = gl::gen_vertex_array();
        let transparentvbo = gl::gen_buffer();
        gl::bind_vertex_array(transparentvao);
        gl::bind_buffer(GL_ARRAY_BUFFER, transparentvbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &transparent_vertices, GL_STATIC_DRAW);
        gl::enable_vertex_attrib_array(0);
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(1);
        gl::vertex_attrib_pointer_offset(1, 2, GL_FLOAT, false, stride, 3 * mem::size_of::<f32>() as u32);
        gl::bind_vertex_array(0);

        // load textures

        let cube_texture = load_texture(assets!("textures/marble.jpg"));
        let floor_texture = load_texture(assets!("textures/metal.png"));
        let transparent_texture = load_texture(assets!("textures/window.png"));

        // transparent window locations

        let windows = [
            vec3(-1.5, 0.0, -0.48),
            vec3(1.5, 0.0, 0.51),
            vec3(0.0, 0.0, 0.7),
            vec3(-0.3, 0.0, -2.3),
            vec3(0.5, 0.0, -0.6),
        ];

        // shader configuration

        shader.use_program();
        shader.set_int("texture1", 0);

        (shader, cubevao, planevao, transparentvao, cube_texture, floor_texture, transparent_texture, windows)
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

            // sort the transparent windows before rendering

            windows.sort_by(|a, b| {
                // NOTE: probably not the most efficient way to sort
                let pos = camera.position.to_vec();
                (pos.distance2(*a)).partial_cmp(&pos.distance2(*b)).unwrap().reverse()
            });

            gl::clear_color(0.1, 0.1, 0.1, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            // draw objects
            shader.use_program();
            let projection: Matrix4<f32> =
                perspective(Deg(camera.zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            let view = camera.get_view_matrix();
            let mut model: Matrix4<f32>;
            shader.set_mat4("projection", &projection);
            shader.set_mat4("view", &view);
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
            // windows (from furthest to nearest)
            gl::bind_vertex_array(transparentvao);
            gl::bind_texture(GL_TEXTURE_2D, transparent_texture);
            for v in &windows {
                let model = Matrix4::from_translation(*v);
                shader.set_mat4("model", &model);
                gl::draw_arrays(GL_TRIANGLES, 0, 6);
            }

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
    // gl::delete_vertex_arrays(1, &transparentvao);
    // gl::delete_buffers(1, &cubevbo);
    // gl::delete_buffers(1, &planevbo);
    // gl::delete_buffers(1, &transparentvbo);
}

/// utility function for loading a 2D texture from file
/// NOTE: not the version from common.rs, slightly adapted for this tutorial

pub fn load_texture(path: PathBuf) -> u32 {
    let texture_id = gl::gen_texture();
    let path = path.as_path();

    let img = image::open(&path).expect("Texture failed to load");
    let format = match img {
        ImageLuma8(_) => GL_RED,
        ImageLumaA8(_) => GL_RG,
        ImageRgb8(_) => GL_RGB,
        ImageRgba8(_) => GL_RGBA,
        _ => panic!("unhandled image format"),
    };

    let data = img.as_bytes();

    gl::bind_texture(GL_TEXTURE_2D, texture_id);
    gl::tex_image_2d(
        GL_TEXTURE_2D,
        0,
        format as i32,
        img.width() as i32,
        img.height() as i32,
        0,
        format,
        GL_UNSIGNED_BYTE,
        &data,
    );
    gl::generate_mipmap(GL_TEXTURE_2D);

    gl::tex_parameteri(
        GL_TEXTURE_2D,
        GL_TEXTURE_WRAP_S,
        if format == GL_RGBA { GL_CLAMP_TO_EDGE } else { GL_REPEAT } as i32,
    ); // for this tutorial: use GL_CLAMP_TO_EDGE to prevent semi-transparent borders. Due to interpolation it takes texels from next repeat );
    gl::tex_parameteri(
        GL_TEXTURE_2D,
        GL_TEXTURE_WRAP_T,
        if format == GL_RGBA { GL_CLAMP_TO_EDGE } else { GL_REPEAT } as i32,
    );
    gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR as i32);
    gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);

    texture_id
}
