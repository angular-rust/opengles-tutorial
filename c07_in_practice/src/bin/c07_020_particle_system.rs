// This is an example that demonstrates rendering a particle system
// using a vertex shader and point sprites. Ported from
// OpenGL(R) ES 2.0 Programming Guide - Chapter 13

#![allow(dead_code)]
#![allow(unused_variables)]
use cgmath::Point3;
use dx::{
    app::{runner::Runner, Application, Key, Store, Time},
    assets, color, glchk,
    gles::{core20::gl, enums::*, utils},
    utils::get_time,
};
use image::GenericImageView;
use rand::random;
use spin_sleep::LoopHelper;
use std::{mem, time::SystemTime};

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

    println!("tex {} {} {}", tex_id, width, height);
    gl::tex_image_2d(GL_TEXTURE_2D, 0, GL_RGB as i32, width, height, 0, GL_RGB, GL_UNSIGNED_BYTE, &data);

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
    vbo: u32,

    loop_helper: LoopHelper,

    // Handle to a program object
    program_id: u32,

    // Attribute locations
    lifetime_loc: i32,
    start_position_loc: i32,
    end_position_loc: i32,

    // Uniform location
    time_loc: i32,
    color_loc: i32,
    center_position_loc: i32,
    sampler_loc: i32,

    // Texture handle
    texture_id: u32,

    // Particle vertex data
    particles: Vec<Particle>,

    // Current time
    time: f32,

    start_time: SystemTime,
}

impl App {
    //  Update time-based variables
    fn update(&mut self, delta_time: f32) {
        self.time += delta_time;

        if self.time >= 1.0 {
            self.time = 0.0;

            // Pick a new start location and color
            let x = random::<f32>() - 0.5;
            let y = random::<f32>() - 0.5;
            let z = random::<f32>() - 0.5;

            let center = [x, y, z];
            gl::uniform3fv(self.center_position_loc, &center);
            glchk!("uniform3fv");

            // Random color
            let red = random::<f32>() / 2.0 + 0.5;
            let green = random::<f32>() / 2.0 + 0.5;
            let blue = random::<f32>() / 2.0 + 0.5;
            let alpha = 0.5;

            let color = [red, green, blue, alpha];
            gl::uniform4fv(self.color_loc, &color);
            glchk!("uniform4fv");
        }

        // Load uniform time variable
        gl::uniform1f(self.time_loc, self.time);
        glchk!("uniform1f");
    }
}

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
            utils::shader_from_file(assets!("shaders/2.0.particle.vs").to_str().unwrap(), GL_VERTEX_SHADER).unwrap();
        let fragment_shader =
            utils::shader_from_file(assets!("shaders/2.0.particle.fs").to_str().unwrap(), GL_FRAGMENT_SHADER).unwrap();
        let program_id = utils::program_from_shaders(vertex_shader, fragment_shader).unwrap();

        // Get the attribute locations
        let lifetime_loc = gl::get_attrib_location(program_id, "a_lifetime");
        glchk!("get_attrib_location");

        let start_position_loc = gl::get_attrib_location(program_id, "a_startPosition");
        glchk!("get_attrib_location");

        let end_position_loc = gl::get_attrib_location(program_id, "a_endPosition");
        glchk!("get_attrib_location");

        // Get the uniform locations
        let time_loc = gl::get_uniform_location(program_id, "u_time");
        glchk!("get_uniform_location");

        let center_position_loc = gl::get_uniform_location(program_id, "u_centerPosition");
        glchk!("get_uniform_location");

        let color_loc = gl::get_uniform_location(program_id, "u_color");
        glchk!("get_uniform_location");

        let sampler_loc = gl::get_uniform_location(program_id, "s_texture");
        glchk!("get_uniform_location");

        let mut particles = Vec::new();

        // Fill in particle data array
        for idx in 0..NUM_PARTICLES {
            // Lifetime of particle
            let lifetime = random::<f32>();

            // End position of particle
            let end_position =
                Point3::new(random::<f32>() * 3.0 - 1.5, random::<f32>() * 3.0 - 1.5, random::<f32>() * 3.0 - 1.5);

            // Start position of particle
            let start_position = Point3::new(
                random::<f32>() / 3.0 - 0.125,
                random::<f32>() / 3.0 - 0.125,
                random::<f32>() / 3.0 - 0.125,
            );

            particles.push(Particle {
                lifetime,
                start_position,
                end_position,
            })
        }

        // Initialize time to cause reset on first update
        let time = 1.0;

        let texture_id = load_texture("textures/smoke.tga");

        // int VBO
        let vbo = gl::gen_buffer();
        glchk!("gen_buffer");

        gl::bind_buffer(GL_ARRAY_BUFFER, vbo);
        glchk!("bind_buffer");

        gl::buffer_data(GL_ARRAY_BUFFER, particles.as_slice(), GL_DYNAMIC_DRAW);
        glchk!("buffer_data");

        let loop_helper = spin_sleep::LoopHelper::builder().build_with_target_rate(250.0);

        Ok(App {
            vbo,

            program_id,

            loop_helper,

            lifetime_loc,
            start_position_loc,
            end_position_loc,
            time_loc,
            center_position_loc,
            color_loc,
            sampler_loc,
            texture_id,
            time,
            particles,
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

        let current_frame = get_time(&self.start_time) as f32;
        self.update(0.01);

        // bind VBO
        gl::bind_buffer(GL_ARRAY_BUFFER, self.vbo);
        glchk!("bind_buffer");

        let fsize = mem::size_of::<f32>() as i32;

        // Load the vertex attributes
        gl::vertex_attrib_pointer_offset(self.lifetime_loc as u32, 1, GL_FLOAT, false, PARTICLE_SIZE * fsize, 0);
        glchk!("vertex_attrib_pointer_offset");

        gl::enable_vertex_attrib_array(self.lifetime_loc as u32);
        glchk!("enable_vertex_attrib_array");

        gl::vertex_attrib_pointer_offset(
            self.end_position_loc as u32,
            3,
            GL_FLOAT,
            false,
            PARTICLE_SIZE * fsize,
            1 * fsize as u32,
        );
        glchk!("vertex_attrib_pointer_offset");

        gl::enable_vertex_attrib_array(self.end_position_loc as u32);
        glchk!("enable_vertex_attrib_array");

        gl::vertex_attrib_pointer_offset(
            self.start_position_loc as u32,
            3,
            GL_FLOAT,
            false,
            PARTICLE_SIZE * fsize,
            4 * fsize as u32,
        );
        glchk!("vertex_attrib_pointer_offset");

        gl::enable_vertex_attrib_array(self.start_position_loc as u32);
        glchk!("enable_vertex_attrib_array");

        // Blend particles
        gl::enable(GL_BLEND);
        glchk!("enable");

        gl::blend_func(GL_SRC_ALPHA, GL_ONE);
        glchk!("blend_func");

        // Bind the texture
        gl::active_texture(GL_TEXTURE0);
        glchk!("active_texture");

        gl::bind_texture(GL_TEXTURE_2D, self.texture_id);
        glchk!("bind_texture");

        // Set the sampler texture unit to 0
        gl::uniform1i(self.sampler_loc, 0);
        glchk!("uniform1i");

        gl::draw_arrays(GL_POINTS, 0, NUM_PARTICLES as i32);
        glchk!("draw_arrays");

        if let Some(rate) = self.loop_helper.report_rate() {
            runner.set_title(&format!("{} {:.0} FPS", "ParticleSystem", rate));
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
        // Delete texture object
        gl::delete_textures(&[self.texture_id]);

        // Delete program object
        gl::delete_program(self.program_id);
    }
}

fn main() {
    env_logger::init();
    Runner::run::<App>("ParticleSystem", 640, 480, ());
}
