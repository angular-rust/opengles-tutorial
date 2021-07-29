use cgmath::{Matrix4, Vector3};
use dx::gles::{core30::gl, enums::*};
use std::{fs::File, io::Read, path::PathBuf, str};

pub struct Shader {
    pub id: u32,
}

/// NOTE: mixture of `shader_s.h` and `shader_m.h` (the latter just contains
/// a few more setters for uniforms)
#[allow(dead_code)]
impl Shader {
    pub fn new<T>(vertex_path: T, fragment_path: T) -> Self
    where
        T: Into<PathBuf>,
    {
        let mut shader = Shader { id: 0 };

        // 1. retrieve the vertex/fragment source code from filesystem
        let vertex_path: PathBuf = vertex_path.into();
        let mut v_shader_file =
            File::open(vertex_path.as_path()).unwrap_or_else(|_| panic!("Failed to open {:?}", vertex_path.as_path()));

        let fragment_path: PathBuf = fragment_path.into();
        let mut f_shader_file = File::open(fragment_path.as_path())
            .unwrap_or_else(|_| panic!("Failed to open {:?}", fragment_path.as_path()));

        let mut vertex_code = String::new();
        let mut fragment_code = String::new();

        v_shader_file
            .read_to_string(&mut vertex_code)
            .expect("Failed to read vertex shader");
        f_shader_file
            .read_to_string(&mut fragment_code)
            .expect("Failed to read fragment shader");

        // 2. compile shaders

        // vertex shader
        let vertex = gl::create_shader(GL_VERTEX_SHADER);
        gl::shader_source(vertex, vertex_code.as_bytes());
        gl::compile_shader(vertex);
        shader.check_compile_errors(vertex, "VERTEX");
        // fragment Shader
        let fragment = gl::create_shader(GL_FRAGMENT_SHADER);
        gl::shader_source(fragment, fragment_code.as_bytes());
        gl::compile_shader(fragment);
        shader.check_compile_errors(fragment, "FRAGMENT");
        // shader Program
        let id = gl::create_program();
        gl::attach_shader(id, vertex);
        gl::attach_shader(id, fragment);
        gl::link_program(id);
        shader.check_compile_errors(id, "PROGRAM");
        // delete the shaders as they're linked into our program now and no longer necessary
        gl::delete_shader(vertex);
        gl::delete_shader(fragment);
        shader.id = id;

        shader
    }

    /// activate the shader

    pub fn use_program(&self) {
        gl::use_program(self.id)
    }

    /// utility uniform functions

    pub fn set_bool(&self, name: &str, value: bool) {
        gl::uniform1i(gl::get_uniform_location(self.id, name), value as i32);
    }

    pub fn set_int(&self, name: &str, value: i32) {
        gl::uniform1i(gl::get_uniform_location(self.id, name), value);
    }

    pub fn set_float(&self, name: &str, value: f32) {
        gl::uniform1f(gl::get_uniform_location(self.id, name), value);
    }

    pub fn set_vector3(&self, name: &str, value: &Vector3<f32>) {
        let value: &[f32; 3] = value.as_ref();
        gl::uniform3fv(gl::get_uniform_location(self.id, name), value);
    }

    pub fn set_vec3(&self, name: &str, x: f32, y: f32, z: f32) {
        gl::uniform3f(gl::get_uniform_location(self.id, name), x, y, z);
    }

    pub fn set_mat4(&self, name: &str, mat: &Matrix4<f32>) {
        let value: &[f32; 16] = mat.as_ref();
        gl::uniform_matrix4fv(gl::get_uniform_location(self.id, name), false, value);
    }

    /// utility function for checking shader compilation/linking errors.

    fn check_compile_errors(&self, shader: u32, type_: &str) {
        if type_ != "PROGRAM" {
            let success = gl::get_shaderiv(shader, GL_COMPILE_STATUS);
            if success == 0 {
                let len = gl::get_shaderiv(shader, GL_INFO_LOG_LENGTH);

                match gl::get_shader_info_log(shader, len) {
                    Some(message) => {
                        println!("ERROR::SHADER_COMPILATION_ERROR of type: {}\n{}\n ", type_, message);
                    }
                    None => {
                        println!("ERROR::SHADER_COMPILATION_ERROR of type: {}\n ", type_,);
                    }
                };
            }
        } else {
            let success = gl::get_programiv(shader, GL_LINK_STATUS);

            if success == 0 {
                let len = gl::get_programiv(shader, GL_INFO_LOG_LENGTH);

                match gl::get_program_info_log(shader, len) {
                    Some(message) => {
                        println!("ERROR::PROGRAM_LINKING_ERROR of type: {}\n{}\n ", type_, message);
                    }
                    None => {
                        println!("ERROR::PROGRAM_LINKING_ERROR of type: {}\n ", type_);
                    }
                };
            }
        }
    }

    /// Only used in 4.9 Geometry shaders - ignore until then (shader.h in original C++)
    pub fn with_geometry_shader<T>(vertex_path: T, fragment_path: T, geometry_path: T) -> Self
    where
        T: Into<PathBuf>,
    {
        let mut shader = Shader { id: 0 };

        // 1. retrieve the vertex/fragment source code from filesystem
        let vertex_path: PathBuf = vertex_path.into();
        let mut v_shader_file =
            File::open(vertex_path.as_path()).unwrap_or_else(|_| panic!("Failed to open {:?}", vertex_path.as_path()));

        let fragment_path: PathBuf = fragment_path.into();
        let mut f_shader_file = File::open(fragment_path.as_path())
            .unwrap_or_else(|_| panic!("Failed to open {:?}", fragment_path.as_path()));

        let geometry_path: PathBuf = geometry_path.into();
        let mut g_shader_file = File::open(geometry_path.as_path())
            .unwrap_or_else(|_| panic!("Failed to open {:?}", geometry_path.as_path()));

        let mut vertex_code = String::new();
        let mut fragment_code = String::new();
        let mut geometry_code = String::new();

        v_shader_file
            .read_to_string(&mut vertex_code)
            .expect("Failed to read vertex shader");
        f_shader_file
            .read_to_string(&mut fragment_code)
            .expect("Failed to read fragment shader");
        g_shader_file
            .read_to_string(&mut geometry_code)
            .expect("Failed to read geometry shader");

        // 2. compile shaders

        // vertex shader
        let vertex = gl::create_shader(GL_VERTEX_SHADER);
        gl::shader_source(vertex, vertex_code.as_bytes());
        gl::compile_shader(vertex);
        shader.check_compile_errors(vertex, "VERTEX");
        // fragment Shader
        let fragment = gl::create_shader(GL_FRAGMENT_SHADER);
        gl::shader_source(fragment, fragment_code.as_bytes());
        gl::compile_shader(fragment);
        shader.check_compile_errors(fragment, "FRAGMENT");
        // geometry shader
        let geometry = gl::create_shader(GL_GEOMETRY_SHADER);
        gl::shader_source(geometry, geometry_code.as_bytes());
        gl::compile_shader(geometry);
        shader.check_compile_errors(geometry, "GEOMETRY");

        // shader Program
        let id = gl::create_program();
        gl::attach_shader(id, vertex);
        gl::attach_shader(id, fragment);
        gl::attach_shader(id, geometry);
        gl::link_program(id);
        shader.check_compile_errors(id, "PROGRAM");
        // delete the shaders as they're linked into our program now and no longer necessary
        gl::delete_shader(vertex);
        gl::delete_shader(fragment);
        gl::delete_shader(geometry);
        shader.id = id;

        shader
    }
}
