use c06_pbr::{load_texture, process_events, process_input, Camera, Shader};
use cgmath::prelude::*;
use cgmath::{perspective, vec2, vec3, Deg, Matrix4, Point3, Vector3};
use dx::{
    assets,
    gles::{core30::gl, enums::*},
    utils::*,
    Instance,
};
use std::{f32::consts::PI, mem::size_of, time::SystemTime};
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

    let (shader, albedo, normal, metallic, roughness, ao) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);

        // build and compile shaders
        let shader = Shader::new(assets!("shaders/1.2.pbr.vs"), assets!("shaders/1.2.pbr.fs"));

        shader.use_program();
        shader.set_int("albedoMap", 0);
        shader.set_int("normalMap", 1);
        shader.set_int("metallicMap", 2);
        shader.set_int("roughnessMap", 3);
        shader.set_int("aoMap", 4);

        // load PBR material textures

        let albedo = load_texture(assets!("textures/pbr/rusted_iron/albedo.png"));
        let normal = load_texture(assets!("textures/pbr/rusted_iron/normal.png"));
        let metallic = load_texture(assets!("textures/pbr/rusted_iron/metallic.png"));
        let roughness = load_texture(assets!("textures/pbr/rusted_iron/roughness.png"));
        let ao = load_texture(assets!("textures/pbr/rusted_iron/ao.png"));

        // initialize static shader uniforms before rendering

        let projection: Matrix4<f32> = perspective(Deg(camera.zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
        shader.set_mat4("projection", &projection);

        (shader, albedo, normal, metallic, roughness, ao)
    };

    // lights

    let light_positions: [Vector3<f32>; 1] = [vec3(0.0, 10.0, 10.0)];
    let light_colors: [Vector3<f32>; 1] = [vec3(150.0, 150.0, 150.0)];
    let nr_rows = 7;
    let nr_columns = 7;
    let spacing = 2.5;

    let mut spherevao = 0;
    let mut index_count = 0;

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
                    // per-frame time logic
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
            // Set the viewport

            // per-frame time logic

            gl::clear_color(0.1, 0.1, 0.1, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            shader.use_program();
            let view = camera.get_view_matrix();
            shader.set_mat4("view", &view);
            shader.set_vector3("camPos", &camera.position.to_vec());

            gl::active_texture(GL_TEXTURE0);
            gl::bind_texture(GL_TEXTURE_2D, albedo);
            gl::active_texture(GL_TEXTURE1);
            gl::bind_texture(GL_TEXTURE_2D, normal);
            gl::active_texture(GL_TEXTURE2);
            gl::bind_texture(GL_TEXTURE_2D, metallic);
            gl::active_texture(GL_TEXTURE3);
            gl::bind_texture(GL_TEXTURE_2D, roughness);
            gl::active_texture(GL_TEXTURE4);
            gl::bind_texture(GL_TEXTURE_2D, ao);

            // render rows*column number of spheres with varying metallic/roughness values scaled by rows and columns respectively
            let mut model: Matrix4<f32>;
            for row in 0..nr_rows {
                shader.set_float("metallic", row as i32 as f32 / nr_rows as f32);
                for col in 0..nr_columns {
                    // we clamp the roughness to 0.025 - 1.0 as perfectly smooth surfaces (roughness of 0.0) tend to look a bit off
                    // on direct lighting.
                    shader.set_float("roughness", num::clamp(col as i32 as f32 / nr_columns as f32, 0.05, 1.0));

                    let model = Matrix4::from_translation(vec3(
                        (col - (nr_columns / 2)) as f32 * spacing,
                        (row - (nr_rows / 2)) as f32 * spacing,
                        0.0,
                    ));
                    shader.set_mat4("model", &model);
                    render_sphere(&mut spherevao, &mut index_count);
                }
            }

            // render light source (simply re-render sphere at light positions)
            // this looks a bit off as we use the same shader, but it'll make their positions obvious and
            // keeps the codeprint small.
            for (i, light_position) in light_positions.iter().enumerate() {
                // NOTE: toggle comments on next two lines to animate the lights
                // let newPos = lightPosition + vec3((glfw.get_time() as f32 * 5.0).sin() * 5.0, 0.0, 0.0);
                let new_pos = *light_position;
                let mut name = format!("lightPositions[{}]", i);
                shader.set_vector3(&name, &new_pos);
                name = format!("lightColors[{}]", i);
                shader.set_vector3(&name, &light_colors[i]);

                model = Matrix4::from_translation(new_pos);
                model = model * Matrix4::from_scale(0.5);
                shader.set_mat4("model", &model);
                render_sphere(&mut spherevao, &mut index_count);
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

pub fn render_sphere(spherevao: &mut u32, index_count: &mut u32) {
    if *spherevao == 0 {
        *spherevao = gl::gen_vertex_array();

        let vbo = gl::gen_buffer();
        let ebo = gl::gen_buffer();

        let mut positions = vec![];
        let mut uv = vec![];
        let mut normals = vec![];
        let mut indices = vec![];

        const X_SEGMENTS: u32 = 64;
        const Y_SEGMENTS: u32 = 64;
        for y in 0..Y_SEGMENTS + 1 {
            for x in 0..X_SEGMENTS + 1 {
                let x_segment = x as f32 / X_SEGMENTS as f32;
                let y_segment = y as f32 / Y_SEGMENTS as f32;
                let x_pos = (x_segment * 2.0 * PI).cos() * (y_segment * PI).sin();
                let y_pos = (y_segment * PI).cos();
                let z_pos = (x_segment * 2.0 * PI).sin() * (y_segment * PI).sin();

                positions.push(vec3(x_pos, y_pos, z_pos));
                uv.push(vec2(x_segment, y_segment));
                normals.push(vec3(x_pos, y_pos, z_pos));
            }
        }

        let mut odd_row = false;
        for y in 0..Y_SEGMENTS {
            if odd_row {
                // even rows: y == 0, y == 2; and so on
                for x in 0..X_SEGMENTS + 1 {
                    indices.push(y * (X_SEGMENTS + 1) + x);
                    indices.push((y + 1) * (X_SEGMENTS + 1) + x);
                }
            } else {
                for x in (0..X_SEGMENTS + 1).rev() {
                    indices.push((y + 1) * (X_SEGMENTS + 1) + x);
                    indices.push(y * (X_SEGMENTS + 1) + x);
                }
            }
            odd_row = !odd_row;
        }
        *index_count = indices.len() as u32;

        let mut data: Vec<f32> = Vec::new();
        for (i, position) in positions.iter().enumerate() {
            data.push(position.x);
            data.push(position.y);
            data.push(position.z);
            if !uv.is_empty() {
                data.push(uv[i].x);
                data.push(uv[i].y);
            }
            if !normals.is_empty() {
                data.push(normals[i].x);
                data.push(normals[i].y);
                data.push(normals[i].z);
            }
        }
        gl::bind_vertex_array(*spherevao);
        gl::bind_buffer(GL_ARRAY_BUFFER, vbo);
        gl::buffer_data(GL_ARRAY_BUFFER, data.as_slice(), GL_STATIC_DRAW);
        gl::bind_buffer(GL_ELEMENT_ARRAY_BUFFER, ebo);
        gl::buffer_data(GL_ELEMENT_ARRAY_BUFFER, indices.as_slice(), GL_STATIC_DRAW);
        let stride = (3 + 2 + 3) * size_of::<f32>() as i32;
        gl::enable_vertex_attrib_array(0);
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(1);
        gl::vertex_attrib_pointer_offset(1, 2, GL_FLOAT, false, stride, 3 * size_of::<f32>() as u32);
        gl::enable_vertex_attrib_array(2);
        gl::vertex_attrib_pointer_offset(2, 3, GL_FLOAT, false, stride, 5 * size_of::<f32>() as u32);
    }

    gl::bind_vertex_array(*spherevao);
    gl::draw_elements_offset(GL_TRIANGLE_STRIP, *index_count as i32, GL_UNSIGNED_INT, 0);
}
