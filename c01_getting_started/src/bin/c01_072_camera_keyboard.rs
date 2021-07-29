use c01_getting_started::Shader;
use cgmath::prelude::*;
use cgmath::{perspective, vec3, Deg, Matrix4, Point3, Vector3};
use dx::{
    assets,
    gles::{core30::gl, enums::*},
    utils::get_time,
    Instance,
};
use image::GenericImageView;
use std::{mem, time::SystemTime};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

// camera
const CAMERA_FRONT: Vector3<f32> = Vector3 {
    x: 0.0,
    y: 0.0,
    z: -1.0,
};

const CAMERA_UP: Vector3<f32> = Vector3 { x: 0.0, y: 1.0, z: 0.0 };

pub fn main() {
    let start_time = SystemTime::now();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut camera_pos = Point3::new(0.0, 0.0, 3.0);

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

    let (our_shader, vao, texture1, texture2, cube_positions) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);

        // build and compile our shader program
        let our_shader = Shader::new(assets!("shaders/7.2.camera.vs"), assets!("shaders/7.2.camera.fs"));

        // set up vertex data (and buffer(s)) and configure vertex attributes

        let vertices: [f32; 180] = [
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
        // world space positions of our cubes
        let cube_positions: [Vector3<f32>; 10] = [
            vec3(0.0, 0.0, 0.0),
            vec3(2.0, 5.0, -15.0),
            vec3(-1.5, -2.2, -2.5),
            vec3(-3.8, -2.0, -12.3),
            vec3(2.4, -0.4, -3.5),
            vec3(-1.7, 3.0, -7.5),
            vec3(1.3, -2.0, -2.5),
            vec3(1.5, 2.0, -2.5),
            vec3(1.5, 0.2, -1.5),
            vec3(-1.3, 1.0, -1.5),
        ];

        let vao = gl::gen_vertex_array();
        let vbo = gl::gen_buffer();

        gl::bind_vertex_array(vao);

        gl::bind_buffer(GL_ARRAY_BUFFER, vbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &vertices, GL_STATIC_DRAW);

        let stride = 5 * mem::size_of::<f32>() as i32;
        // position attribute
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(0);
        // texture coord attribute
        gl::vertex_attrib_pointer_offset(1, 2, GL_FLOAT, false, stride, 3 * mem::size_of::<f32>() as u32);
        gl::enable_vertex_attrib_array(1);

        // load and create a texture

        // texture 1

        let texture1 = gl::gen_texture();
        gl::bind_texture(GL_TEXTURE_2D, texture1);
        // set the texture wrapping parameters
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT as i32); // set texture wrapping to GL_REPEAT (default wrapping method)
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT as i32);
        // set texture filtering parameters
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
        // load image, create texture and generate mipmaps
        let img = image::open(assets!("textures/container.jpg")).expect("Failed to load texture");
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

        // texture 2

        let texture2 = gl::gen_texture();
        gl::bind_texture(GL_TEXTURE_2D, texture2);
        // set the texture wrapping parameters
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT as i32); // set texture wrapping to GL_REPEAT (default wrapping method)
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT as i32);
        // set texture filtering parameters
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
        // load image, create texture and generate mipmaps
        let img = image::open(assets!("textures/awesomeface.png")).expect("Failed to load texture");
        let img = img.flipv(); // flip loaded texture on the y-axis.
        let data = img.as_bytes();
        // note that the awesomeface.png has transparency and thus an alpha channel, so make sure to tell OpenGL the data type is of GL_RGBA
        gl::tex_image_2d(
            GL_TEXTURE_2D,
            0,
            GL_RGB as i32,
            img.width() as i32,
            img.height() as i32,
            0,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            &data,
        );
        gl::generate_mipmap(GL_TEXTURE_2D);

        // tell opengl for each sampler to which texture unit it belongs to (only has to be done once)
        our_shader.use_program();
        our_shader.set_int("texture1", 0);
        our_shader.set_int("texture2", 1);

        // pass projection matrix to shader (as projection matrix rarely changes there's no need to do this per frame)
        let projection: Matrix4<f32> = perspective(Deg(45.0), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
        our_shader.set_mat4("projection", &projection);

        (our_shader, vao, texture1, texture2, cube_positions)
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
                    let current_frame = get_time(&start_time) as f32;

                    // input
                    process_input(&input, current_frame - last_frame, &mut camera_pos);
                    last_frame = current_frame;
                }
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
            // per-frame time logic

            gl::clear_color(0.2, 0.3, 0.3, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            // bind textures on corresponding texture units
            gl::active_texture(GL_TEXTURE0);
            gl::bind_texture(GL_TEXTURE_2D, texture1);
            gl::active_texture(GL_TEXTURE1);
            gl::bind_texture(GL_TEXTURE_2D, texture2);

            // activate shader
            our_shader.use_program();

            // camera/view transformation
            let view: Matrix4<f32> = Matrix4::look_at_rh(camera_pos, camera_pos + CAMERA_FRONT, CAMERA_UP);
            our_shader.set_mat4("view", &view);

            // render boxes
            gl::bind_vertex_array(vao);
            for (i, position) in cube_positions.iter().enumerate() {
                // calculate the model matrix for each object and pass it to shader before drawing
                let mut model: Matrix4<f32> = Matrix4::from_translation(*position);
                let angle = 20.0 * i as f32;
                model = model * Matrix4::from_axis_angle(vec3(1.0, 0.3, 0.5).normalize(), Deg(angle));
                our_shader.set_mat4("model", &model);

                gl::draw_arrays(GL_TRIANGLES, 0, 36);
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

    // // optional: de-allocate all resources once they've outlived their purpose:
    //     gl::delete_vertex_arrays(1, &vao);
    //     gl::delete_buffers(1, &vbo);
}

/// NOTE: not the same function as the one in common.rs!
fn process_input(
    input: &KeyboardInput,
    delta_time: f32,
    camera_pos: &mut Point3<f32>,
    // CAMERA_FRONT: &mut Vector3<f32>,
) {
    let camera_speed = 2.5 * delta_time;
    match input {
        KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::W),
            ..
        } => *camera_pos += camera_speed * CAMERA_FRONT,
        KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::S),
            ..
        } => *camera_pos += -(camera_speed * CAMERA_FRONT),
        KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::A),
            ..
        } => *camera_pos += -(CAMERA_FRONT.cross(CAMERA_UP).normalize() * camera_speed),
        KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::D),
            ..
        } => *camera_pos += CAMERA_FRONT.cross(CAMERA_UP).normalize() * camera_speed,
        _ => {}
    }
}
