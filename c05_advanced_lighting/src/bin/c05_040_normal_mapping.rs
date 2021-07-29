use c05_advanced_lighting::{load_texture, process_events, process_input, Camera, Shader};
use cgmath::prelude::*;
use cgmath::{perspective, vec2, vec3, Deg, Matrix4, Point3, Vector2, Vector3};
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

    // tell to capture our mouse
    // window.set_cursor_mode(CursorMode::Disabled);

    let (shader, diffuse_map, normal_map) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);

        // build and compile shaders
        let shader = Shader::new(assets!("shaders/4.normal_mapping.vs"), assets!("shaders/4.normal_mapping.fs"));

        // load textures

        let diffuse_map = load_texture(assets!("textures/brickwall.jpg"));
        let normal_map = load_texture(assets!("textures/brickwall_normal.jpg"));

        // shader configuration

        shader.use_program();
        shader.set_int("diffuse_map", 0);
        shader.set_int("normalMap", 1);

        (shader, diffuse_map, normal_map)
    };

    // lighting info

    let light_pos: Vector3<f32> = vec3(0.5, 1.0, 0.3);

    let mut quadvao = 0;
    let mut quadvbo = 0;

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
                    // process_input(&input, current_frame - last_frame, &mut camera, &mut gammaEnabled, &mut gammaKeyPressed);
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
            // Set the viewport
            // per-frame time logic

            // let current_frame = get_time(&start_time);
            // delta_time = current_frame - last_frame;
            // last_frame = current_frame;

            // render
            gl::clear_color(0.1, 0.1, 0.1, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            // configure view/projection matrices
            let projection: Matrix4<f32> =
                perspective(Deg(camera.zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            let view = camera.get_view_matrix();
            shader.use_program();
            shader.set_mat4("projection", &projection);
            shader.set_mat4("view", &view);
            // render normal-mapped quad
            let mut model: Matrix4<f32> =
                Matrix4::from_axis_angle(vec3(1.0, 0.0, 1.0).normalize(), Deg(get_time(&start_time) * -10.0)); // rotate the quad to show normal mapping from multiple directions
            shader.set_mat4("model", &model);
            shader.set_vector3("viewPos", &camera.position.to_vec());
            shader.set_vector3("lightPos", &light_pos);
            gl::active_texture(GL_TEXTURE0);
            gl::bind_texture(GL_TEXTURE_2D, diffuse_map);
            gl::active_texture(GL_TEXTURE1);
            gl::bind_texture(GL_TEXTURE_2D, normal_map);
            render_quad(&mut quadvao, &mut quadvbo);

            // render light source (simply re-renders a smaller plane at the light's position for debugging/visualization)
            model = Matrix4::from_translation(light_pos);
            model = model * Matrix4::from_scale(0.1);
            shader.set_mat4("model", &model);
            render_quad(&mut quadvao, &mut quadvbo);

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

// renders a 1x1 quad in NDC with manually calculated tangent vectors
fn render_quad(quadvao: &mut u32, quadvbo: &mut u32) {
    if *quadvao == 0 {
        // positions
        let pos1: Vector3<f32> = vec3(-1.0, 1.0, 0.0);
        let pos2: Vector3<f32> = vec3(-1.0, -1.0, 0.0);
        let pos3: Vector3<f32> = vec3(1.0, -1.0, 0.0);
        let pos4: Vector3<f32> = vec3(1.0, 1.0, 0.0);
        // texture coordinates
        let uv1: Vector2<f32> = vec2(0.0, 1.0);
        let uv2: Vector2<f32> = vec2(0.0, 0.0);
        let uv3: Vector2<f32> = vec2(1.0, 0.0);
        let uv4: Vector2<f32> = vec2(1.0, 1.0);
        // normal vector
        let nm: Vector3<f32> = vec3(0.0, 0.0, 1.0);

        // calculate tangent/bitangent vectors of both triangles
        let mut tangent1: Vector3<f32> = vec3(0.0, 0.0, 0.0);
        let mut bitangent1: Vector3<f32> = vec3(0.0, 0.0, 0.0);
        let mut tangent2: Vector3<f32> = vec3(0.0, 0.0, 0.0);
        let mut bitangent2: Vector3<f32> = vec3(0.0, 0.0, 0.0);
        // triangle 1
        let mut edge1 = pos2 - pos1;
        let mut edge2 = pos3 - pos1;
        let mut delta_uv1 = uv2 - uv1;
        let mut delta_uv2 = uv3 - uv1;

        let mut f = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

        tangent1.x = f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x);
        tangent1.y = f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y);
        tangent1.z = f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z);
        tangent1 = tangent1.normalize();

        bitangent1.x = f * (-delta_uv2.x * edge1.x + delta_uv1.x * edge2.x);
        bitangent1.y = f * (-delta_uv2.x * edge1.y + delta_uv1.x * edge2.y);
        bitangent1.z = f * (-delta_uv2.x * edge1.z + delta_uv1.x * edge2.z);
        bitangent1 = bitangent1.normalize();

        // triangle 2
        edge1 = pos3 - pos1;
        edge2 = pos4 - pos1;
        delta_uv1 = uv3 - uv1;
        delta_uv2 = uv4 - uv1;

        f = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

        tangent2.x = f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x);
        tangent2.y = f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y);
        tangent2.z = f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z);
        tangent2 = tangent2.normalize();

        bitangent2.x = f * (-delta_uv2.x * edge1.x + delta_uv1.x * edge2.x);
        bitangent2.y = f * (-delta_uv2.x * edge1.y + delta_uv1.x * edge2.y);
        bitangent2.z = f * (-delta_uv2.x * edge1.z + delta_uv1.x * edge2.z);
        bitangent2 = bitangent2.normalize();

        let quad_vertices: [f32; 84] = [
            // positions            // normal         // texcoords  // tangent                          // bitangent
            pos1.x,
            pos1.y,
            pos1.z,
            nm.x,
            nm.y,
            nm.z,
            uv1.x,
            uv1.y,
            tangent1.x,
            tangent1.y,
            tangent1.z,
            bitangent1.x,
            bitangent1.y,
            bitangent1.z,
            pos2.x,
            pos2.y,
            pos2.z,
            nm.x,
            nm.y,
            nm.z,
            uv2.x,
            uv2.y,
            tangent1.x,
            tangent1.y,
            tangent1.z,
            bitangent1.x,
            bitangent1.y,
            bitangent1.z,
            pos3.x,
            pos3.y,
            pos3.z,
            nm.x,
            nm.y,
            nm.z,
            uv3.x,
            uv3.y,
            tangent1.x,
            tangent1.y,
            tangent1.z,
            bitangent1.x,
            bitangent1.y,
            bitangent1.z,
            pos1.x,
            pos1.y,
            pos1.z,
            nm.x,
            nm.y,
            nm.z,
            uv1.x,
            uv1.y,
            tangent2.x,
            tangent2.y,
            tangent2.z,
            bitangent2.x,
            bitangent2.y,
            bitangent2.z,
            pos3.x,
            pos3.y,
            pos3.z,
            nm.x,
            nm.y,
            nm.z,
            uv3.x,
            uv3.y,
            tangent2.x,
            tangent2.y,
            tangent2.z,
            bitangent2.x,
            bitangent2.y,
            bitangent2.z,
            pos4.x,
            pos4.y,
            pos4.z,
            nm.x,
            nm.y,
            nm.z,
            uv4.x,
            uv4.y,
            tangent2.x,
            tangent2.y,
            tangent2.z,
            bitangent2.x,
            bitangent2.y,
            bitangent2.z,
        ];

        // configure plane vao
        *quadvao = gl::gen_vertex_array();
        *quadvbo = gl::gen_buffer();
        gl::bind_vertex_array(*quadvao);
        gl::bind_buffer(GL_ARRAY_BUFFER, *quadvbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &quad_vertices, GL_STATIC_DRAW);
        let stride = 14 * mem::size_of::<f32>() as i32;
        gl::enable_vertex_attrib_array(0);
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(1);
        gl::vertex_attrib_pointer_offset(1, 3, GL_FLOAT, false, stride, 3 * mem::size_of::<f32>() as u32);
        gl::enable_vertex_attrib_array(2);
        gl::vertex_attrib_pointer_offset(2, 2, GL_FLOAT, false, stride, 6 * mem::size_of::<f32>() as u32);
        gl::enable_vertex_attrib_array(3);
        gl::vertex_attrib_pointer_offset(3, 3, GL_FLOAT, false, stride, 8 * mem::size_of::<f32>() as u32);
        gl::enable_vertex_attrib_array(4);
        gl::vertex_attrib_pointer_offset(4, 3, GL_FLOAT, false, stride, 11 * mem::size_of::<f32>() as u32);
    }

    gl::bind_vertex_array(*quadvao);
    gl::draw_arrays(GL_TRIANGLES, 0, 6);
    gl::bind_vertex_array(0);
}
