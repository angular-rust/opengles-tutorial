use c01_getting_started::Shader;
use dx::{
    assets,
    gles::{core30::gl, enums::*},
    Instance,
};
use image::GenericImageView;
use std::mem;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// settings
// const SCR_WIDTH: u32 = 800;
// const SCR_HEIGHT: u32 = 600;

pub fn main() {
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

    let (our_shader, vao, texture) = {
        // build and compile our shader program

        let our_shader = Shader::new(assets!("shaders/4.1.texture.vs"), assets!("shaders/4.1.texture.fs"));

        // set up vertex data (and buffer(s)) and configure vertex attributes

        // HINT: type annotation is crucial since default for float literals is f64
        let vertices: [f32; 32] = [
            // positions       // colors        // texture coords
            0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, // top right
            0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, // bottom right
            -0.5, -0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, // bottom left
            -0.5, 0.5, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, // top left
        ];
        let indices = [
            0, 1, 3, // first Triangle
            1, 2, 3, // second Triangle
        ];

        let vao = gl::gen_vertex_array();
        let vbo = gl::gen_buffer();
        let ebo = gl::gen_buffer();

        gl::bind_vertex_array(vao);

        gl::bind_buffer(GL_ARRAY_BUFFER, vbo);
        gl::buffer_data(GL_ARRAY_BUFFER, &vertices, GL_STATIC_DRAW);

        gl::bind_buffer(GL_ELEMENT_ARRAY_BUFFER, ebo);
        gl::buffer_data(GL_ELEMENT_ARRAY_BUFFER, &indices, GL_STATIC_DRAW);

        let stride = 8 * mem::size_of::<f32>() as i32;
        // position attribute
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, stride, 0);
        gl::enable_vertex_attrib_array(0);
        // color attribute
        gl::vertex_attrib_pointer_offset(1, 3, GL_FLOAT, false, stride, 3 * mem::size_of::<f32>() as u32);
        gl::enable_vertex_attrib_array(1);
        // texture coord attribute
        gl::vertex_attrib_pointer_offset(2, 2, GL_FLOAT, false, stride, (6 * mem::size_of::<f32>()) as u32);
        gl::enable_vertex_attrib_array(2);

        // load and create a texture

        let texture = gl::gen_texture();
        gl::bind_texture(GL_TEXTURE_2D, texture); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
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

        (our_shader, vao, texture)
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
            gl::clear(GL_COLOR_BUFFER_BIT);

            // bind Texture
            gl::bind_texture(GL_TEXTURE_2D, texture);

            // render container
            our_shader.use_program();
            gl::bind_vertex_array(vao);
            gl::draw_elements_offset(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);

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
    //     gl::delete_buffers(1, &EBO);
}
