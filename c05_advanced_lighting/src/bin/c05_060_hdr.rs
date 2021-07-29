use c05_advanced_lighting::{process_events, Camera, Camera_Movement::*, Shader};
use cgmath::prelude::*;
use cgmath::{perspective, vec3, Deg, Matrix4, Point3, Vector3};
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

    let mut hdr = true;
    let mut hdr_key_pressed = false;
    let mut exposure: f32 = 1.0;

    let mut camera = Camera {
        position: Point3::new(0.0, 0.0, 5.0),
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

    let mut cubevao = 0;
    let mut cubevbo = 0;
    let (shader, hdr_shader, wood_texture, hdr_fbo, color_buffer, light_positions, light_colors) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);

        // build and compile shaders
        let shader = Shader::new(assets!("shaders/6.lighting.vs"), assets!("shaders/6.lighting.fs"));
        let hdr_shader = Shader::new(assets!("shaders/6.hdr.vs"), assets!("shaders/6.hdr.fs"));

        // load textures

        let wood_texture = load_texture(assets!("textures/wood.png"), true); // note that we're loading the texture as an SRGB texture

        // configure floating point framebuffer

        let hdr_fbo = gl::gen_framebuffer();
        // create floating point color buffer
        let color_buffer = gl::gen_texture();
        gl::bind_texture(GL_TEXTURE_2D, color_buffer);
        gl::empty_tex_image_2d(
            GL_TEXTURE_2D,
            0,
            GL_RGBA16F as i32,
            SCR_WIDTH as i32,
            SCR_HEIGHT as i32,
            0,
            GL_RGBA,
            GL_FLOAT,
        );
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
        gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);

        // create depth buffer (renderbuffer)
        let rbo_depth = gl::gen_renderbuffer();
        gl::bind_renderbuffer(GL_RENDERBUFFER, rbo_depth);
        gl::renderbuffer_storage(GL_RENDERBUFFER, GL_DEPTH_COMPONENT, SCR_WIDTH as i32, SCR_HEIGHT as i32);
        // attach buffers
        gl::bind_framebuffer(GL_FRAMEBUFFER, hdr_fbo);
        gl::framebuffer_texture_2d(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, color_buffer, 0);
        gl::framebuffer_renderbuffer(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_RENDERBUFFER, rbo_depth);
        if gl::check_framebuffer_status(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE {
            println!("Framebuffer not complete!");
        }
        gl::bind_framebuffer(GL_FRAMEBUFFER, 0);

        // lighting info

        // positions
        let mut light_positions: Vec<Vector3<f32>> = Vec::new();
        light_positions.push(vec3(0.0, 0.0, 49.5)); // back light
        light_positions.push(vec3(-1.4, -1.9, 9.0));
        light_positions.push(vec3(0.0, -1.8, 4.0));
        light_positions.push(vec3(0.8, -1.7, 6.0));
        // colors
        let mut light_colors: Vec<Vector3<f32>> = Vec::new();
        light_colors.push(vec3(200.0, 200.0, 200.0));
        light_colors.push(vec3(0.1, 0.0, 0.0));
        light_colors.push(vec3(0.0, 0.0, 0.2));
        light_colors.push(vec3(0.0, 0.1, 0.0));

        // shader configuration

        shader.use_program();
        shader.set_int("diffuseTexture", 0);
        hdr_shader.use_program();
        hdr_shader.set_int("hdrBuffer", 0);

        (shader, hdr_shader, wood_texture, hdr_fbo, color_buffer, light_positions, light_colors)
    };

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
                    process_input(
                        &input,
                        current_frame - last_frame,
                        &mut camera,
                        &mut hdr,
                        &mut hdr_key_pressed,
                        &mut exposure,
                    );
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

            gl::clear_color(0.1, 0.1, 0.1, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            // 1. render scene into floating point framebuffer

            gl::bind_framebuffer(GL_FRAMEBUFFER, hdr_fbo);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
            let projection: Matrix4<f32> =
                perspective(Deg(camera.zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            let view = camera.get_view_matrix();
            shader.use_program();
            shader.set_mat4("projection", &projection);
            shader.set_mat4("view", &view);
            gl::active_texture(GL_TEXTURE0);
            gl::bind_texture(GL_TEXTURE_2D, wood_texture);
            // set lighting uniforms
            for (i, light_pos) in light_positions.iter().enumerate() {
                let name = format!("lights[{}].Position", i);
                shader.set_vector3(&name, light_pos);
                let name = format!("lights[{}].Color", i);
                shader.set_vector3(&name, &light_colors[i]);
            }
            shader.set_vector3("viewPos", &camera.position.to_vec());
            // render tunnel
            let mut model: Matrix4<f32> = Matrix4::from_translation(vec3(0.0, 0.0, 25.0));
            model = model * Matrix4::from_nonuniform_scale(2.5, 2.5, 27.5);
            shader.set_mat4("model", &model);
            shader.set_bool("inverse_normals", true);
            render_cube(&mut cubevao, &mut cubevbo);
            gl::bind_framebuffer(GL_FRAMEBUFFER, 0);

            // 2. now render floating point color buffer to 2D quad and tonemap HDR colors to default framebuffer's (clamped) color range
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
            hdr_shader.use_program();
            gl::active_texture(GL_TEXTURE0);
            gl::bind_texture(GL_TEXTURE_2D, color_buffer);
            hdr_shader.set_bool("hdr", hdr);
            hdr_shader.set_float("exposure", exposure);
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

// renderCube() renders a 1x1 3D cube in NDC.
fn render_cube(cubevao: &mut u32, cubevbo: &mut u32) {
    if *cubevao == 0 {
        let vertices: [f32; 288] = [
            // back face
            -1.0, -1.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, // bottom-left
            1.0, 1.0, -1.0, 0.0, 0.0, -1.0, 1.0, 1.0, // top-right
            1.0, -1.0, -1.0, 0.0, 0.0, -1.0, 1.0, 0.0, // bottom-right
            1.0, 1.0, -1.0, 0.0, 0.0, -1.0, 1.0, 1.0, // top-right
            -1.0, -1.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, // bottom-left
            -1.0, 1.0, -1.0, 0.0, 0.0, -1.0, 0.0, 1.0, // top-left
            // front face
            -1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, // bottom-left
            1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, // bottom-right
            1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, // top-right
            1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, // top-right
            -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, // top-left
            -1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, // bottom-left
            // left face
            -1.0, 1.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0, // top-right
            -1.0, 1.0, -1.0, -1.0, 0.0, 0.0, 1.0, 1.0, // top-left
            -1.0, -1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, // bottom-left
            -1.0, -1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, // bottom-left
            -1.0, -1.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0, // bottom-right
            -1.0, 1.0, 1.0, -1.0, 0.0, 0.0, 1.0, 0.0, // top-right
            // right face
            1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, // top-left
            1.0, -1.0, -1.0, 1.0, 0.0, 0.0, 0.0, 1.0, // bottom-right
            1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 1.0, // top-right
            1.0, -1.0, -1.0, 1.0, 0.0, 0.0, 0.0, 1.0, // bottom-right
            1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, // top-left
            1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, // bottom-left
            // bottom face
            -1.0, -1.0, -1.0, 0.0, -1.0, 0.0, 0.0, 1.0, // top-right
            1.0, -1.0, -1.0, 0.0, -1.0, 0.0, 1.0, 1.0, // top-left
            1.0, -1.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, // bottom-left
            1.0, -1.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, // bottom-left
            -1.0, -1.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, // bottom-right
            -1.0, -1.0, -1.0, 0.0, -1.0, 0.0, 0.0, 1.0, // top-right
            // top face
            -1.0, 1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 1.0, // top-left
            1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, // bottom-right
            1.0, 1.0, -1.0, 0.0, 1.0, 0.0, 1.0, 1.0, // top-right
            1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, // bottom-right
            -1.0, 1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 1.0, // top-left
            -1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, // bottom-left
        ];

        *cubevao = gl::gen_vertex_array();
        *cubevbo = gl::gen_buffer();
        // fill buffer
        gl::bind_buffer(GL_ARRAY_BUFFER, *cubevbo);
        // let size = (vertices.len() * mem::size_of::<f32>()) as isize;
        // let data = &vertices[0];
        gl::buffer_data(GL_ARRAY_BUFFER, &vertices, GL_STATIC_DRAW);
        // link vertex attributes
        gl::bind_vertex_array(*cubevao);
        let stride = 8 * mem::size_of::<f32>() as i32;
        gl::enable_vertex_attrib_array(0);
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(1);
        gl::vertex_attrib_pointer_offset(1, 3, GL_FLOAT, false, stride, 3 * mem::size_of::<f32>() as u32);
        gl::enable_vertex_attrib_array(2);
        gl::vertex_attrib_pointer_offset(2, 2, GL_FLOAT, false, stride, 6 * mem::size_of::<f32>() as u32);
        gl::bind_buffer(GL_ARRAY_BUFFER, 0);
        gl::bind_vertex_array(0);
    }
    // render Cube
    gl::bind_vertex_array(*cubevao);
    gl::draw_arrays(GL_TRIANGLES, 0, 36);
    gl::bind_vertex_array(0);
}

// renders a 1x1 quad in NDC with manually calculated tangent vectors
fn render_quad(quadvao: &mut u32, quadvbo: &mut u32) {
    if *quadvao == 0 {
        let quad_vertices: [f32; 20] = [
            // positions     // texture Coords
            -1.0, 1.0, 0.0, 0.0, 1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 0.0,
        ];

        // setup plane vao
        *quadvao = gl::gen_vertex_array();
        *quadvbo = gl::gen_buffer();
        gl::bind_vertex_array(*quadvao);
        gl::bind_buffer(GL_ARRAY_BUFFER, *quadvbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &quad_vertices, GL_STATIC_DRAW);
        let stride = 5 * mem::size_of::<f32>() as i32;
        gl::enable_vertex_attrib_array(0);
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(1);
        gl::vertex_attrib_pointer_offset(1, 2, GL_FLOAT, false, stride, 3 * mem::size_of::<f32>() as u32);
    }
    gl::bind_vertex_array(*quadvao);
    gl::draw_arrays(GL_TRIANGLE_STRIP, 0, 4);
    gl::bind_vertex_array(0);
}

// NOTE: not the same version as in common.rs
pub fn process_input(
    input: &KeyboardInput,
    delta_time: f32,
    camera: &mut Camera,
    hdr: &mut bool,
    hdr_key_pressed: &mut bool,
    exposure: &mut f32,
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
            virtual_keycode: Some(VirtualKeyCode::Space),
            ..
        } => {
            if !(*hdr_key_pressed) {
                *hdr = !(*hdr);
                *hdr_key_pressed = true;
                println!("hdr: {} | exposure: {}", if *hdr { "on" } else { "off" }, *exposure);
            }
        }
        KeyboardInput {
            state: ElementState::Released,
            virtual_keycode: Some(VirtualKeyCode::Space),
            ..
        } => {
            *hdr_key_pressed = false;
        }
        KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::Q),
            ..
        } => {
            if *exposure > 0.0 {
                *exposure -= 0.01;
            } else {
                *exposure = 0.0;
            }
            println!("hdr: {} | exposure: {}", if *hdr { "on" } else { "off" }, *exposure);
        }
        KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::E),
            ..
        } => {
            *exposure += 0.01;
            println!("hdr: {} | exposure: {}", if *hdr { "on" } else { "off" }, *exposure);
        }
        _ => {}
    }
}

// NOTE: not the same version as in common.rs
pub fn load_texture(path: PathBuf, gamma_correction: bool) -> u32 {
    let texture_id = gl::gen_texture();
    let path = path.as_path();

    let img = image::open(&path).expect("Texture failed to load");
    // need two different formats for gamma correction
    let (internal_format, data_format) = match img {
        ImageLuma8(_) => (GL_RED, GL_RED),
        ImageLumaA8(_) => (GL_RG, GL_RG),
        ImageRgb8(_) => (if gamma_correction { GL_SRGB } else { GL_RGB }, GL_RGB),
        // ImageRgba8(_) => (if gammaCorrection { GL_SRGB_ALPHA } else { GL_RGB }, GL_RGBA),
        _ => panic!("unhandled image format"),
    };

    let data = img.as_bytes();

    gl::bind_texture(GL_TEXTURE_2D, texture_id);
    gl::tex_image_2d(
        GL_TEXTURE_2D,
        0,
        internal_format as i32,
        img.width() as i32,
        img.height() as i32,
        0,
        data_format,
        GL_UNSIGNED_BYTE,
        &data,
    );
    gl::generate_mipmap(GL_TEXTURE_2D);

    gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT as i32);
    gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT as i32);
    gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR as i32);
    gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);

    texture_id
}
