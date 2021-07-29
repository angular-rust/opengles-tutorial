use c04_advanced_opengl::{process_events, process_input, Camera, Model, Shader};
use cgmath::prelude::*;
use cgmath::{perspective, vec3, Deg, Matrix4, Point3};
use dx::{
    assets,
    gles::{core30::gl, enums::*},
    utils::*,
    Instance,
};
use rand::Rng;
use std::time::SystemTime;
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
        position: Point3::new(0.0, 0.0, 55.0),
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

    let (shader, rock, planet, model_matrices) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);

        // build and compile shaders
        let shader = Shader::new(assets!("shaders/10.2.instancing.vs"), assets!("shaders/10.2.instancing.fs"));

        // load models

        let rock = Model::new(assets!("objects/rock/rock.obj"));
        let planet = Model::new(assets!("objects/planet/planet.obj"));

        // generate a large list of semi-random model transformation matrices

        let amount = 1000;
        let mut model_matrices: Vec<Matrix4<f32>> = Vec::with_capacity(amount);
        let mut rng = rand::thread_rng();
        let radius = 50.0;
        let offset: f32 = 2.5;
        for i in 0..amount {
            let angle = i as i32 as f32 / amount as f32 * 360.0;
            let mut displacement = (rng.gen::<i32>() % (2.0 * offset * 100.0) as i32) as f32 / 100.0 - offset;
            let x = angle.sin() * radius + displacement;
            displacement = (rng.gen::<i32>() % (2.0 * offset * 100.0) as i32) as f32 / 100.0 - offset;
            let y = displacement * 0.4; // keep height of asteroid field smaller compared to width of x and z
            displacement = (rng.gen::<i32>() % (2.0 * offset * 100.0) as i32) as f32 / 100.0 - offset;
            let z = angle.cos() * radius + displacement;
            let mut model = Matrix4::<f32>::from_translation(vec3(x, y, z));

            // 2. scale: Scale between 0.05 and 0.25
            let scale = (rng.gen::<i32>() % 20) as f32 / 100.0 + 0.05;
            model = model * Matrix4::from_scale(scale);

            // 3. rotation: add random rotation around a (semi)randomly picked rotation axis vector
            let rot_angle = (rng.gen::<i32>() % 360) as f32;
            model = model * Matrix4::from_axis_angle(vec3(0.4, 0.6, 0.8).normalize(), Deg(rot_angle));

            // 4. now add to list of matrices
            model_matrices.push(model);
        }

        (shader, rock, planet, model_matrices)
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

            // configure transformation matrices
            let projection: Matrix4<f32> =
                perspective(Deg(camera.zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 1000.0);
            let view = camera.get_view_matrix();
            shader.use_program();
            shader.set_mat4("projection", &projection);
            shader.set_mat4("view", &view);

            // draw planet
            let mut model = Matrix4::<f32>::from_translation(vec3(0.0, -3.0, 0.0));
            model = model * Matrix4::from_scale(4.0);
            shader.set_mat4("model", &model);
            planet.draw(&shader);

            // draw meteorites
            for model in &model_matrices {
                shader.set_mat4("model", model);
                rock.draw(&shader);
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
}
