use c05_advanced_lighting::{load_texture, process_events, Camera, Camera_Movement::*, Shader};
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

    let mut blinn = false;
    let mut blinn_key_pressed = false;

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

    let (shader, planevao, floor_texture) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);
        gl::enable(GL_BLEND);
        gl::blend_func(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

        // build and compile shaders
        let shader = Shader::new(assets!("shaders/1.advanced_lighting.vs"), assets!("shaders/1.advanced_lighting.fs"));

        // set up vertex data (and buffer(s)) and configure vertex attributes

        let plane_vertices: [f32; 48] = [
            // positions         // normals      // texcoords
            10.0, -0.5, 10.0, 0.0, 1.0, 0.0, 10.0, 0.0, -10.0, -0.5, 10.0, 0.0, 1.0, 0.0, 0.0, 0.0, -10.0, -0.5, -10.0,
            0.0, 1.0, 0.0, 0.0, 10.0, 10.0, -0.5, 10.0, 0.0, 1.0, 0.0, 10.0, 0.0, -10.0, -0.5, -10.0, 0.0, 1.0, 0.0,
            0.0, 10.0, 10.0, -0.5, -10.0, 0.0, 1.0, 0.0, 10.0, 10.0,
        ];
        // plane vao

        let planevao = gl::gen_vertex_array();
        let planevbo = gl::gen_buffer();
        gl::bind_vertex_array(planevao);
        gl::bind_buffer(GL_ARRAY_BUFFER, planevbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &plane_vertices, GL_STATIC_DRAW);
        gl::enable_vertex_attrib_array(0);
        let stride = 8 * mem::size_of::<f32>() as i32;
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(1);
        gl::vertex_attrib_pointer_offset(1, 3, GL_FLOAT, false, stride, 3 * mem::size_of::<f32>() as u32);
        gl::enable_vertex_attrib_array(2);
        gl::vertex_attrib_pointer_offset(2, 2, GL_FLOAT, false, stride, 6 * mem::size_of::<f32>() as u32);
        gl::bind_vertex_array(0);

        // load textures

        let floor_texture = load_texture(assets!("textures/wood.png"));

        // shader configuration

        shader.use_program();
        shader.set_int("texture1", 0);

        (shader, planevao, floor_texture)
    };

    // lighting info

    let light_pos = vec3(0.0, 0.0, 0.0);

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
                    process_input(&input, current_frame - last_frame, &mut camera, &mut blinn, &mut blinn_key_pressed);
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

            // let current_frame = get_time(&start_time);
            // delta_time = current_frame - last_frame;
            // last_frame = current_frame;

            gl::clear_color(0.1, 0.1, 0.1, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            // draw objects
            shader.use_program();
            let projection: Matrix4<f32> =
                perspective(Deg(camera.zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            let view = camera.get_view_matrix();
            shader.set_mat4("projection", &projection);
            shader.set_mat4("view", &view);
            // set light uniforms
            shader.set_vector3("viewPos", &camera.position.to_vec());
            shader.set_vector3("lightPos", &light_pos);
            shader.set_int("blinn", blinn as i32);
            // floor
            gl::bind_vertex_array(planevao);
            gl::active_texture(GL_TEXTURE0);
            gl::bind_texture(GL_TEXTURE_2D, floor_texture);
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
    // gl::delete_vertex_arrays(1, &planevao);
    // gl::delete_buffers(1, &planevbo);
}

// NOTE: not the same version as in common.rs
pub fn process_input(
    input: &KeyboardInput,
    delta_time: f32,
    camera: &mut Camera,
    blinn: &mut bool,
    blinn_key_pressed: &mut bool,
) {
    match input {
        KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::W),
            ..
        } => camera.process_keyboard(FORWARD, delta_time),
        KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::S),
            ..
        } => camera.process_keyboard(BACKWARD, delta_time),
        KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::A),
            ..
        } => camera.process_keyboard(LEFT, delta_time),
        KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::D),
            ..
        } => camera.process_keyboard(RIGHT, delta_time),
        KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::B),
            ..
        } => {
            if !(*blinn_key_pressed) {
                *blinn = !(*blinn);
                *blinn_key_pressed = true;
                println!("{}", if *blinn { "Blinn-Phong" } else { "Phong" })
            }
        }
        KeyboardInput {
            state: ElementState::Released,
            virtual_keycode: Some(VirtualKeyCode::B),
            ..
        } => {
            *blinn_key_pressed = false;
        }
        _ => {}
    }
}
