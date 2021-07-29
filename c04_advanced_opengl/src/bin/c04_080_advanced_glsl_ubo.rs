use c04_advanced_opengl::{process_events, process_input, Camera, Shader};
// use cgmath::prelude::*;
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
    let (device, _queue) = adapter.request_device();
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

    let (shader_red, shader_green, shader_blue, shader_yellow, _cubevbo, cubevao, ubo_matrices) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);

        // build and compile shaders
        let shader_red = Shader::new(assets!("shaders/8.advanced_glsl.vs"), assets!("shaders/8.red.fs"));
        let shader_green = Shader::new(assets!("shaders/8.advanced_glsl.vs"), assets!("shaders/8.green.fs"));
        let shader_blue = Shader::new(assets!("shaders/8.advanced_glsl.vs"), assets!("shaders/8.blue.fs"));
        let shader_yellow = Shader::new(assets!("shaders/8.advanced_glsl.vs"), assets!("shaders/8.yellow.fs"));

        // set up vertex data (and buffer(s)) and configure vertex attributes

        let cube_yertices: [f32; 108] = [
            // positions
            -0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, -0.5,
            -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5,
            -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
            0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, -0.5, -0.5, -0.5, 0.5, -0.5,
            -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5,
            0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5,
        ];

        // cube vao
        let cubevao = gl::gen_vertex_array();
        let cubevbo = gl::gen_buffer();
        gl::bind_vertex_array(cubevao);
        gl::bind_buffer(GL_ARRAY_BUFFER, cubevbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &cube_yertices, GL_STATIC_DRAW);
        let stride = 3 * mem::size_of::<f32>() as i32;
        gl::enable_vertex_attrib_array(0);
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);

        // configure a uniform buffer object

        // first. We get the relevant block indices
        let uniform_block_index_red = gl::get_uniform_block_index(shader_red.id, "Matrices");
        let uniform_block_index_green = gl::get_uniform_block_index(shader_green.id, "Matrices");
        let uniform_block_index_blue = gl::get_uniform_block_index(shader_blue.id, "Matrices");
        let uniform_block_index_yellow = gl::get_uniform_block_index(shader_yellow.id, "Matrices");
        // then we link each shader's uniform block to this uniform binding point
        gl::uniform_block_binding(shader_red.id, uniform_block_index_red, 0);
        gl::uniform_block_binding(shader_green.id, uniform_block_index_green, 0);
        gl::uniform_block_binding(shader_blue.id, uniform_block_index_blue, 0);
        gl::uniform_block_binding(shader_yellow.id, uniform_block_index_yellow, 0);

        // Now actually create the buffer
        let ubo_matrices = gl::gen_buffer();
        gl::bind_buffer(GL_UNIFORM_BUFFER, ubo_matrices);
        gl::buffer_data_size(GL_UNIFORM_BUFFER, 0, GL_STATIC_DRAW);
        // define the range of the buffer that links to a uniform binding point
        gl::bind_buffer_range(GL_UNIFORM_BUFFER, 0, ubo_matrices, 0, 2 * 2 * mem::size_of::<Matrix4<f32>>() as isize);

        // store the projection matrix (we only do this once now) (note: we're not using zoom anymore by changing the FoV)
        let projection: Matrix4<f32> = perspective(Deg(45.0), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
        gl::bind_buffer(GL_UNIFORM_BUFFER, ubo_matrices);
        gl::buffer_sub_data(GL_UNIFORM_BUFFER, 0, &[projection]);
        gl::bind_buffer(GL_UNIFORM_BUFFER, 0);

        (shader_red, shader_green, shader_blue, shader_yellow, cubevbo, cubevao, ubo_matrices)
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

            // let current_frame = get_time(&start_time);
            // last_frame = current_frame;

            gl::clear_color(0.1, 0.1, 0.1, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            // set the view and projection matrix in the uniform block - we only have to do this once per loop iteration.
            let view = camera.get_view_matrix();
            gl::bind_buffer(GL_UNIFORM_BUFFER, ubo_matrices);
            let size = mem::size_of::<Matrix4<f32>>() as isize;
            gl::buffer_sub_data(GL_UNIFORM_BUFFER, size, &[view]);
            gl::bind_buffer(GL_UNIFORM_BUFFER, 0);

            // draw 4 cubes
            // RED
            gl::bind_vertex_array(cubevao);
            shader_red.use_program();
            let mut model = Matrix4::from_translation(vec3(-0.75, 0.75, 0.0)); // move top-left
            shader_red.set_mat4("model", &model);
            gl::draw_arrays(GL_TRIANGLES, 0, 36);
            // GREEN
            shader_green.use_program();
            model = Matrix4::from_translation(vec3(0.75, 0.75, 0.0)); // move top-right
            shader_green.set_mat4("model", &model);
            gl::draw_arrays(GL_TRIANGLES, 0, 36);
            // YELLOW
            shader_yellow.use_program();
            model = Matrix4::from_translation(vec3(-0.75, -0.75, 0.0)); // move bottom-left
            shader_yellow.set_mat4("model", &model);
            gl::draw_arrays(GL_TRIANGLES, 0, 36);
            // BLUE
            shader_blue.use_program();
            model = Matrix4::from_translation(vec3(0.75, -0.75, 0.0)); // move bottom-right
            shader_blue.set_mat4("model", &model);
            gl::draw_arrays(GL_TRIANGLES, 0, 36);

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
    // gl::delete_buffers(1, &cubevbo);
}
