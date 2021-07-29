use c01_getting_started::Shader;
use cgmath::prelude::*;
use cgmath::{perspective, vec3, Deg, Matrix4, Rad};
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

pub fn main() {
    let start_time = SystemTime::now();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

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

    let (our_shader, vao, texture1, texture2) = {
        // configure global opengl state

        gl::enable(GL_DEPTH_TEST);

        // build and compile our shader program
        let our_shader =
            Shader::new(assets!("shaders/6.2.coordinate_systems.vs"), assets!("shaders/6.2.coordinate_systems.fs"));

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

        (our_shader, vao, texture1, texture2)
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
                _ => {}
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
            gl::clear_color(0.2, 0.3, 0.3, 1.0);
            gl::clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

            // bind textures on corresponding texture units
            gl::active_texture(GL_TEXTURE0);
            gl::bind_texture(GL_TEXTURE_2D, texture1);
            gl::active_texture(GL_TEXTURE1);
            gl::bind_texture(GL_TEXTURE_2D, texture2);

            // activate shader
            our_shader.use_program();

            // create transformations
            // NOTE: cgmath requires axis vectors to be normalized!
            let model: Matrix4<f32> =
                Matrix4::from_axis_angle(vec3(0.5, 1.0, 0.0).normalize(), Rad(get_time(&start_time)));
            let view: Matrix4<f32> = Matrix4::from_translation(vec3(0., 0., -3.));
            let projection: Matrix4<f32> = perspective(Deg(45.0), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            // retrieve the matrix uniform locations
            let model_loc = gl::get_uniform_location(our_shader.id, "model");
            let view_loc = gl::get_uniform_location(our_shader.id, "view");
            // pass them to the shaders (3 different ways)
            gl::uniform_matrix4fv(model_loc, false, AsRef::<[f32; 16]>::as_ref(&model));
            gl::uniform_matrix4fv(view_loc, false, AsRef::<[f32; 16]>::as_ref(&view));
            // note: currently we set the projection matrix each frame, but since the projection matrix rarely changes it's often best practice to set it outside the main loop only once.
            our_shader.set_mat4("projection", &projection);

            // render container
            gl::bind_vertex_array(vao);
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

    // // optional: de-allocate all resources once they've outlived their purpose:
    //     gl::delete_vertex_arrays(1, &vao);
    //     gl::delete_buffers(1, &vbo);
}
