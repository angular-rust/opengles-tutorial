use c04_advanced_opengl::{process_events, process_input, Camera, Model, Shader};
use cgmath::prelude::*;
use cgmath::{perspective, vec3, Deg, Matrix4, Point3, Vector4};
use dx::{
    assets,
    gles::{core30::gl, enums::*},
    utils::*,
    Instance,
};
use rand::Rng;
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
        position: Point3::new(0.0, 0.0, 155.0),
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

    #[allow(unused_variables)]
    let (asteroid_shader, planet_shader, rock, planet, amount) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);

        // build and compile shaders
        let asteroid_shader = Shader::new(assets!("shaders/10.3.asteroids.vs"), assets!("shaders/10.3.asteroids.fs"));
        let planet_shader = Shader::new(assets!("shaders/10.3.planet.vs"), assets!("shaders/10.3.planet.fs"));

        // load models

        let rock = Model::new(assets!("objects/rock/rock.obj"));
        let planet = Model::new(assets!("objects/planet/planet.obj"));

        // generate a large list of semi-random model transformation matrices

        let amount = 100_000;
        let mut model_matrices: Vec<Matrix4<f32>> = Vec::with_capacity(amount);
        let mut rng = rand::thread_rng();
        let radius = 150.0;
        let offset: f32 = 25.0;
        for idx in 0..amount {
            let angle = idx as i32 as f32 / amount as f32 * 360.0;
            let mut displacement = (rng.gen::<u32>() % (2.0 * offset * 100.0) as u32) as f32 / 100.0 - offset;
            let x = angle.sin() * radius + displacement;
            displacement = (rng.gen::<u32>() % (2.0 * offset * 100.0) as u32) as f32 / 100.0 - offset;
            let y = displacement * 0.4; // keep height of asteroid field smaller compared to width of x and z
            displacement = (rng.gen::<u32>() % (2.0 * offset * 100.0) as u32) as f32 / 100.0 - offset;
            let z = angle.cos() * radius + displacement;
            let mut model = Matrix4::<f32>::from_translation(vec3(x, y, z));

            // 2. scale: Scale between 0.05 and 0.25
            let scale = (rng.gen::<u32>() % 20) as f32 / 100.0 + 0.05;
            model = model * Matrix4::from_scale(scale);

            // 3. rotation: add random rotation around a (semi)randomly picked rotation axis vector
            let rot_angle = (rng.gen::<u32>() % 360) as f32;
            model = model * Matrix4::from_axis_angle(vec3(0.4, 0.6, 0.8).normalize(), Deg(rot_angle));

            // 4. now add to list of matrices
            model_matrices.push(model);
        }

        // configure instanced array

        let buffer = gl::gen_buffer();
        gl::bind_buffer(GL_ARRAY_BUFFER, buffer);
        gl::buffer_data(GL_ARRAY_BUFFER, &model_matrices.as_slice(), GL_STATIC_DRAW);

        // set transformation matrices as an instance vertex attribute (with divisor 1)
        // note: we're cheating a little by taking the, now publicly declared, vao of the model's mesh(es) and adding new vertexAttribPointers
        // normally you'd want to do this in a more organized fashion, but for learning purposes this will do.
        let size_mat4 = mem::size_of::<Matrix4<f32>>() as i32;
        let size_vec4 = mem::size_of::<Vector4<f32>>() as u32;
        for mesh in &rock.meshes {
            let vao = mesh.vao;
            gl::bind_vertex_array(vao);
            // set attribute pointers for matrix (4 times vec4)
            gl::enable_vertex_attrib_array(3);
            gl::vertex_attrib_pointer_offset(3, 4, GL_FLOAT, false, size_mat4, 0);
            gl::enable_vertex_attrib_array(4);
            gl::vertex_attrib_pointer_offset(4, 4, GL_FLOAT, false, size_mat4, size_vec4);
            gl::enable_vertex_attrib_array(5);
            gl::vertex_attrib_pointer_offset(5, 4, GL_FLOAT, false, size_mat4, 2 * size_vec4);
            gl::enable_vertex_attrib_array(6);
            gl::vertex_attrib_pointer_offset(6, 4, GL_FLOAT, false, size_mat4, 3 * size_vec4);

            gl::vertex_attrib_divisor(3, 1);
            gl::vertex_attrib_divisor(4, 1);
            gl::vertex_attrib_divisor(5, 1);
            gl::vertex_attrib_divisor(6, 1);

            gl::bind_vertex_array(0);
        }

        (asteroid_shader, planet_shader, rock, planet, amount)
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
            let projection: Matrix4<f32> = perspective(Deg(45.0), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 1000.0);
            let view = camera.get_view_matrix();
            asteroid_shader.use_program();
            asteroid_shader.set_mat4("projection", &projection);
            asteroid_shader.set_mat4("view", &view);
            planet_shader.use_program();
            planet_shader.set_mat4("projection", &projection);
            planet_shader.set_mat4("view", &view);

            // draw planet
            let mut model = Matrix4::<f32>::from_translation(vec3(0.0, -3.0, 0.0));
            model = model * Matrix4::from_scale(4.0);
            planet_shader.set_mat4("model", &model);
            planet.draw(&planet_shader);

            // draw meteorites
            asteroid_shader.use_program();
            asteroid_shader.set_int("texture_diffuse1", 0);
            gl::active_texture(GL_TEXTURE0);
            gl::bind_texture(GL_TEXTURE_2D, rock.textures_loaded[0].id); // note: we also made the textures_loaded vector public (instead of private) from the model class.

            for mesh in &rock.meshes {
                gl::bind_vertex_array(mesh.vao);
                gl::draw_elements_instanced_offset(
                    GL_TRIANGLES,
                    mesh.indices.len() as i32,
                    GL_UNSIGNED_INT,
                    0,
                    amount as i32,
                );
                gl::bind_vertex_array(0);
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
