use c02_lighting::{load_texture, process_events, process_input, Camera, Shader};
use cgmath::prelude::*;
use cgmath::{perspective, vec3, Deg, Matrix4, Point3};
use dx::{
    assets,
    gles::{core30::gl, enums::*},
    utils::get_time,
    Instance,
};
use std::{mem, time::SystemTime};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

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

    // lighting
    let light_pos = vec3(1.2, 1.0, 2.0);

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

    let (lighting_shader, lamp_shader, cubevao, lightvao, diffuse_map) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);

        // build and compile our shader program
        let lighting_shader =
            Shader::new(assets!("shaders/4.1.lighting_maps.vs"), assets!("shaders/4.1.lighting_maps.fs"));
        let lamp_shader = Shader::new(assets!("shaders/4.1.lamp.vs"), assets!("shaders/4.1.lamp.fs"));

        // set up vertex data (and buffer(s)) and configure vertex attributes

        let vertices: [f32; 288] = [
            // positions       // normals        // texture coords
            -0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 0.0, 0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 0.0, 0.5, 0.5, -0.5, 0.0,
            0.0, -1.0, 1.0, 1.0, 0.5, 0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 1.0, -0.5, 0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 1.0,
            -0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 0.0, -0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, -0.5, 0.5, 0.0,
            0.0, 1.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 1.0, 0.5, 0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 1.0, -0.5,
            0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 1.0, -0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0, -0.5, 0.5, 0.5, -1.0, 0.0,
            0.0, 1.0, 0.0, -0.5, 0.5, -0.5, -1.0, 0.0, 0.0, 1.0, 1.0, -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, 0.0, 1.0, -0.5,
            -0.5, -0.5, -1.0, 0.0, 0.0, 0.0, 1.0, -0.5, -0.5, 0.5, -1.0, 0.0, 0.0, 0.0, 0.0, -0.5, 0.5, 0.5, -1.0, 0.0,
            0.0, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 1.0, 0.0, 0.5, 0.5, -0.5, 1.0, 0.0, 0.0, 1.0, 1.0, 0.5, -0.5,
            -0.5, 1.0, 0.0, 0.0, 0.0, 1.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.0, 1.0, 0.5, -0.5, 0.5, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 1.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.0, 1.0, 0.5, -0.5,
            -0.5, 0.0, -1.0, 0.0, 1.0, 1.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 1.0, 0.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0,
            1.0, 0.0, -0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 0.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.0, 1.0, -0.5, 0.5,
            -0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 1.0, 1.0, 0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 1.0,
            0.0, 0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 1.0, 0.0, -0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 0.0, -0.5, 0.5, -0.5, 0.0,
            1.0, 0.0, 0.0, 1.0,
        ];
        // first, configure the cube's vao (and vbo)

        let cubevao = gl::gen_vertex_array();
        let vbo = gl::gen_buffer();

        gl::bind_buffer(GL_ARRAY_BUFFER, vbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &vertices, GL_STATIC_DRAW);

        gl::bind_vertex_array(cubevao);
        let stride = 8 * mem::size_of::<f32>() as i32;
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(0);
        gl::vertex_attrib_pointer_offset(1, 3, GL_FLOAT, false, stride, 3 * mem::size_of::<f32>() as u32);
        gl::enable_vertex_attrib_array(1);
        gl::vertex_attrib_pointer_offset(2, 2, GL_FLOAT, false, stride, 6 * mem::size_of::<f32>() as u32);
        gl::enable_vertex_attrib_array(2);

        // second, configure the light's vao (vbo stays the same; the vertices are the same for the light object which is also a 3D cube)
        let lightvao = gl::gen_vertex_array();
        gl::bind_vertex_array(lightvao);

        gl::bind_buffer(GL_ARRAY_BUFFER, vbo);

        // note that we update the lamp's position attribute's stride to reflect the updated buffer data
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(0);

        // load textures (we now use a utility function to keep the code more organized)

        let diffuse_map = load_texture(assets!("textures/container2.png"));

        // shader configuration

        lighting_shader.use_program();
        lighting_shader.set_int("material.diffuse", 0);

        (lighting_shader, lamp_shader, cubevao, lightvao, diffuse_map)
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

            // be sure to activate shader when setting uniforms/drawing objects
            lighting_shader.use_program();
            lighting_shader.set_vector3("light.position", &light_pos);
            lighting_shader.set_vector3("viewPos", &camera.position.to_vec());

            // light properties
            lighting_shader.set_vec3("light.ambient", 0.2, 0.2, 0.2);
            lighting_shader.set_vec3("light.diffuse", 0.5, 0.5, 0.5);
            lighting_shader.set_vec3("light.specular", 1.0, 1.0, 1.0);

            // material properties
            lighting_shader.set_vec3("material.specular", 0.5, 0.5, 0.5); // specular lighting doesn't have full effect on this object's material
            lighting_shader.set_float("material.shininess", 64.0);

            // view/projection transformations
            let projection: Matrix4<f32> =
                perspective(Deg(camera.zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            let view = camera.get_view_matrix();
            lighting_shader.set_mat4("projection", &projection);
            lighting_shader.set_mat4("view", &view);

            // world transformation
            let mut model = Matrix4::<f32>::identity();
            lighting_shader.set_mat4("model", &model);

            // bind diffuse map
            gl::active_texture(GL_TEXTURE0);
            gl::bind_texture(GL_TEXTURE_2D, diffuse_map);

            // render the cube
            gl::bind_vertex_array(cubevao);
            gl::draw_arrays(GL_TRIANGLES, 0, 36);

            // also draw the lamp object
            lamp_shader.use_program();
            lamp_shader.set_mat4("projection", &projection);
            lamp_shader.set_mat4("view", &view);
            model = Matrix4::from_translation(light_pos);
            model = model * Matrix4::from_scale(0.2); // a smaller cube
            lamp_shader.set_mat4("model", &model);

            gl::bind_vertex_array(lightvao);
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
    // gl::delete_vertex_arrays(1, &lightvao);
    // gl::delete_buffers(1, &vbo);
}
