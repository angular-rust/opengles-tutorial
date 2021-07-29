// Demonstrates rendering a terrain with vertex texture fetch
// Ported from OpenGL(R) ES 3.0 Programming Guide, 2nd Edition
// OpenGL(R) ES 2.0 Programming Guide - Chapter 14

#![allow(dead_code)]
#![allow(unused_variables)]
use cgmath::prelude::*;
use cgmath::{Deg, Matrix4, Point3, Vector3};
use dx::{
    app::{runner::Runner, Application, Key, Store, Time},
    assets, color, glchk,
    gles::{core20::gl, enums::*, utils},
};
use image::GenericImageView;
use spin_sleep::LoopHelper;
use std::{mem, time::SystemTime};

const POSITION_LOC: usize = 0;

/// \brief Generates a square grid consisting of triangles.  
///        Allocates memory for the vertex data and stores
///        the results in the arrays.  Generate index list as TRIANGLES.
/// @size create a grid of size by size (number of triangles = (size-1)*(size-1)*2)
///
/// Return value: (vertices, indices) vertices, will contain array of float3 positions,
/// indices, will contain the array of indices for the triangle strip
fn gen_square_grid(size: u32) -> (Vec<Point3<f32>>, Vec<u32>) {
    let num_indices = (size - 1) * (size - 1) * 2 * 3;

    let num_vertices = size * size;
    let step_size = size as f32 - 1_f32;

    let mut vertices = Vec::with_capacity(3 * num_vertices as usize);

    for row in 0..size {
        for column in 0..size {
            vertices.push(Point3::new(row as f32 / step_size, column as f32 / step_size, 0_f32));
        }
    }

    // Generate the indices
    let mut indices: Vec<u32> = Vec::with_capacity(num_indices as usize);
    for row in 0..size - 1 {
        for column in 0..size - 1 {
            // two triangles per quad
            indices.push(column + row * size);
            indices.push(column + row * size + 1);
            indices.push(column + (row + 1) * size + 1);
            indices.push(column + row * size);
            indices.push(column + (row + 1) * size + 1);
            indices.push(column + (row + 1) * size);
        }
    }

    return (vertices, indices);
}

// Initialize the MVP matrix
fn init_mvp() -> Matrix4<f32> {
    let width = 640;
    let height = 480;
    // Compute the window aspect ratio
    let aspect = width as f32 / height as f32;

    // Generate a perspective matrix with a 60 degree FOV
    // let perspective = cgmath::perspective(Deg(60.0_f32), aspect, 0.1_f32, 1000.0_f32);

    // Generate a model view matrix to rotate/translate the terrain
    let mut modelview = Matrix4::<f32>::identity();

    // Center the terrain
    modelview = modelview
        * Matrix4::from_translation(Vector3 {
            x: -0.4_f32,
            y: -0.5_f32,
            z: -0.7_f32,
        });

    // Rotate
    modelview = modelview * Matrix4::from_axis_angle(Vector3::unit_x(), Deg(45.0_f32));

    modelview = modelview * Matrix4::from_axis_angle(Vector3::unit_z(), Deg(10.0_f32));

    // Compute the final MVP by multiplying the
    // modelview and perspective matrices together
    // perspective * modelview
    modelview
}

const NUM_PARTICLES: usize = 1000;
const PARTICLE_SIZE: i32 = 7;

// Load texture from disk
fn load_texture(filename: &str) -> u32 {
    let img = image::open(assets!(filename)).expect("Failed to load texture");
    let width = img.width() as i32;
    let height = img.height() as i32;

    let data = img.as_bytes();

    let tex_id = gl::gen_texture();
    gl::bind_texture(GL_TEXTURE_2D, tex_id);

    gl::tex_image_2d(GL_TEXTURE_2D, 0, GL_ALPHA as i32, width, height, 0, GL_ALPHA, GL_UNSIGNED_BYTE, &data);

    gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
    gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
    gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE as i32);
    gl::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE as i32);

    return tex_id;
}

#[repr(C)]
struct Particle {
    lifetime: f32,
    start_position: Point3<f32>,
    end_position: Point3<f32>,
}

struct App {
    // VBOs
    indices_ibo: u32,
    position_vbo: u32,

    loop_helper: LoopHelper,

    // Handle to a program object
    program_id: u32,

    // Attribute locations
    // we do not store position location coz we use layout scheme in GLSL

    // Uniform location
    mvp_loc: i32,
    light_direction_loc: i32,

    // Sampler location
    sampler_loc: i32,

    // Texture handle
    texture_id: u32,

    // MVP matrix
    mvp_matrix: Matrix4<f32>,

    // Number of indices
    num_indices: u32,

    start_time: SystemTime,
}

impl App {}

impl Application<Runner> for App {
    type Context = ();

    type Error = ();

    fn init(
        runner: &mut Runner,
        store: &mut Store<Self::Context, Key>,
        context: &mut Self::Context,
    ) -> Result<Self, Self::Error> {
        let start_time = SystemTime::now();

        let bg = color::GRAY_9;

        // set clear color
        gl::clear_color(bg.red, bg.green, bg.blue, bg.alpha);

        // Load the shaders and get a linked program object
        let vertex_shader =
            utils::shader_from_file(assets!("shaders/3.0.terrain.vs").to_str().unwrap(), GL_VERTEX_SHADER).unwrap();
        let fragment_shader =
            utils::shader_from_file(assets!("shaders/3.0.terrain.fs").to_str().unwrap(), GL_FRAGMENT_SHADER).unwrap();
        let program_id = utils::program_from_shaders(vertex_shader, fragment_shader).unwrap();

        // Get the uniform locations
        let mvp_loc = gl::get_uniform_location(program_id, "u_mvpMatrix");
        let light_direction_loc = gl::get_uniform_location(program_id, "u_lightDirection");

        // Get the sampler location
        let sampler_loc = gl::get_uniform_location(program_id, "s_texture");

        // Generate the position and indices of a square grid for the base terrain
        let grid_size = 200;
        let (positions, indices) = gen_square_grid(grid_size);

        // Index buffer for base terrain
        let indices_ibo = gl::gen_buffer();
        gl::bind_buffer(GL_ELEMENT_ARRAY_BUFFER, indices_ibo);
        gl::buffer_data(GL_ELEMENT_ARRAY_BUFFER, indices.as_slice(), GL_STATIC_DRAW);
        gl::bind_buffer(GL_ELEMENT_ARRAY_BUFFER, 0);

        // Position VBO for base terrain
        let position_vbo = gl::gen_buffer();
        gl::bind_buffer(GL_ARRAY_BUFFER, position_vbo);
        gl::buffer_data(GL_ARRAY_BUFFER, positions.as_slice(), GL_STATIC_DRAW);

        // Load the heightmap
        let texture_id = load_texture("textures/heightmap.tga");

        let loop_helper = spin_sleep::LoopHelper::builder().build_with_target_rate(250.0);

        Ok(App {
            indices_ibo,
            position_vbo,
            mvp_loc,
            light_direction_loc,
            sampler_loc,
            num_indices: indices.len() as u32,
            program_id,
            loop_helper,
            texture_id,
            mvp_matrix: Matrix4::identity(),
            start_time,
        })
    }

    fn resized(&mut self, runner: &mut Runner, context: &mut Self::Context, width: u32, height: u32) {
        // set the viewport
        gl::viewport(0, 0, width as i32, height as i32);
    }

    fn render(&mut self, runner: &mut Runner, context: &mut Self::Context, t: Time) -> bool {
        // clear the color buffer
        gl::clear(GL_COLOR_BUFFER_BIT);
        glchk!("clear");

        // use the program object
        gl::use_program(self.program_id);
        glchk!("use_program");

        self.mvp_matrix = init_mvp();

        // Load the vertex position
        gl::bind_buffer(GL_ARRAY_BUFFER, self.position_vbo);
        gl::vertex_attrib_pointer_offset(POSITION_LOC as u32, 3, GL_FLOAT, false, 3 * mem::size_of::<f32>() as i32, 0);
        gl::enable_vertex_attrib_array(POSITION_LOC as u32);

        // Bind the index buffer
        gl::bind_buffer(GL_ELEMENT_ARRAY_BUFFER, self.indices_ibo);

        // Bind the height map
        gl::active_texture(GL_TEXTURE0);
        gl::bind_texture(GL_TEXTURE_2D, self.texture_id);

        // Load the MVP matrix
        let proj_view: &[f32; 16] = self.mvp_matrix.as_ref();
        gl::uniform_matrix4fv(self.mvp_loc, false, proj_view);

        // Load the light direction
        gl::uniform3f(self.light_direction_loc, 0.86_f32, 0.14_f32, 0.49_f32);

        // Set the height map sampler to texture unit to 0
        gl::uniform1i(self.sampler_loc, 0);

        // Draw the grid
        gl::draw_elements_offset(GL_TRIANGLES, self.num_indices as i32, GL_UNSIGNED_INT, 0);

        //   let current_frame = get_time(&self.start_time) as f32;
        //   self.update(0.01);

        if let Some(rate) = self.loop_helper.report_rate() {
            runner.set_title(&format!("{} {:.0} FPS", "TerrainRendering", rate));
        }

        self.loop_helper.loop_sleep();
        self.loop_helper.loop_start();

        // request redraw
        true
    }
}

impl Drop for App {
    // cleanup
    fn drop(&mut self) {
        // Delete VBO & IBO
        gl::delete_buffers(&[self.indices_ibo, self.position_vbo]);

        // Delete texture object
        gl::delete_textures(&[self.texture_id]);

        // Delete program object
        gl::delete_program(self.program_id);
    }
}

fn main() {
    env_logger::init();
    Runner::run::<App>("TerrainRendering", 640, 480, ());
}
