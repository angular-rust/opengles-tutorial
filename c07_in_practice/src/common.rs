use std::path::PathBuf;

/// Common code that the original tutorials repeat over and over and over and over
use dx::gles::{core30::gl, enums::*};
use image::{DynamicImage::*, GenericImageView};
use winit::event::*;

use super::camera::Camera;
use super::camera::Camera_Movement::*;

/// Event processing function as introduced in 1.7.4 (Camera Class) and used in
/// most later tutorials
pub fn process_events(
    event: &WindowEvent,
    first_mouse: &mut bool,
    last_x: &mut f32,
    last_y: &mut f32,
    camera: &mut Camera,
) {
    match event {
        WindowEvent::Resized(physical_size) => {
            // make sure the viewport matches the new window dimensions; note that width and
            // height will be significantly larger than specified on retina displays.
            gl::viewport(0, 0, physical_size.width as i32, physical_size.height as i32);
        }
        WindowEvent::CursorMoved { position, .. } => {
            let (xpos, ypos) = (position.x as f32, position.y as f32);
            if *first_mouse {
                *last_x = xpos;
                *last_y = ypos;
                *first_mouse = false;
            }

            let xoffset = xpos - *last_x;
            let yoffset = *last_y - ypos; // reversed since y-coordinates go from bottom to top

            *last_x = xpos;
            *last_y = ypos;

            camera.process_mouse_movement(xoffset, yoffset, true);
        }
        WindowEvent::MouseWheel {
            delta: MouseScrollDelta::PixelDelta(ph),
            ..
        } => {
            let yoffset = ph.y as f32;
            camera.process_mouse_scroll(yoffset as f32);
        }
        WindowEvent::MouseWheel {
            delta: MouseScrollDelta::LineDelta(_rows, lines),
            ..
        } => {
            let yoffset = lines * 3.0;
            camera.process_mouse_scroll(yoffset as f32);
        }
        _ => {}
    }
}

/// Input processing function as introduced in 1.7.4 (Camera Class) and used in
/// most later tutorials
pub fn process_input(input: &KeyboardInput, delta_time: f32, camera: &mut Camera) {
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
        _ => {}
    }
}

/// utility function for loading a 2D texture from file

#[allow(dead_code)]
pub fn load_texture<T>(path: T) -> u32
where
    T: Into<PathBuf>,
{
    let texture_id = gl::gen_texture();
    let path = path.into();
    let img = image::open(&path).expect("Texture failed to load");

    let format = match img {
        ImageLuma8(_) => GL_RED,
        ImageLumaA8(_) => GL_RG,
        ImageRgb8(_) => GL_RGB,
        ImageRgba8(_) => GL_RGBA,
        _ => panic!("unhandled image format"),
    };

    gl::bind_texture(GL_TEXTURE_2D, texture_id);
    gl::tex_image_2d(
        GL_TEXTURE_2D,
        0,
        format as i32,
        img.width() as i32,
        img.height() as i32,
        0,
        format,
        GL_UNSIGNED_BYTE,
        img.as_bytes(),
    );
    gl::generate_mipmap(GL_TEXTURE_2D);

    gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT as i32);
    gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT as i32);
    gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR as i32);
    gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);

    texture_id
}
