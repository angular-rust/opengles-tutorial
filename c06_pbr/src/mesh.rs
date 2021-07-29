use cgmath::prelude::*;
use cgmath::{Vector2, Vector3};
use dx::gles::{core30::gl, enums::*};
use std::mem::size_of;

use super::shader::Shader;

// NOTE: without repr(C) the compiler may reorder the fields or use different padding/alignment than C.
// Depending on how you pass the data to OpenGL, this may be bad. In this case it's not strictly
// necessary though because of the `offset!` macro used below in setupMesh()
#[repr(C)]
pub struct Vertex {
    // position
    pub position: Vector3<f32>,
    // normal
    pub normal: Vector3<f32>,
    // texCoords
    pub tex_coords: Vector2<f32>,
    // tangent
    pub tangent: Vector3<f32>,
    // bitangent
    pub bitangent: Vector3<f32>,
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Vector3::zero(),
            normal: Vector3::zero(),
            tex_coords: Vector2::zero(),
            tangent: Vector3::zero(),
            bitangent: Vector3::zero(),
        }
    }
}

#[derive(Clone)]
pub struct Texture {
    pub id: u32,
    pub type_: String,
    pub path: String,
}

pub struct Mesh {
    /*  Mesh Data  */
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Texture>,
    pub vao: u32,

    /*  Render data  */
    vbo: u32,
    ebo: u32,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Texture>) -> Mesh {
        let mut mesh = Mesh {
            vertices,
            indices,
            textures,
            vao: 0,
            vbo: 0,
            ebo: 0,
        };

        // now that we have all the required data, set the vertex buffers and its attribute pointers.
        mesh.setup_mesh();

        mesh
    }

    /// render the mesh
    pub fn draw(&self, shader: &Shader) {
        // bind appropriate textures
        let mut diffuse_nr = 0;
        let mut specular_nr = 0;
        let mut normal_nr = 0;
        let mut height_nr = 0;
        for (i, texture) in self.textures.iter().enumerate() {
            gl::active_texture(GL_TEXTURE0 + i as u32); // active proper texture unit before binding
                                                        // retrieve texture number (the N in diffuse_textureN)
            let name = &texture.type_;
            let number = match name.as_str() {
                "texture_diffuse" => {
                    diffuse_nr += 1;
                    diffuse_nr
                }
                "texture_specular" => {
                    specular_nr += 1;
                    specular_nr
                }
                "texture_normal" => {
                    normal_nr += 1;
                    normal_nr
                }
                "texture_height" => {
                    height_nr += 1;
                    height_nr
                }
                _ => panic!("unknown texture type"),
            };
            // now set the sampler to the correct texture unit
            let sampler = format!("{}{}", name, number);
            gl::uniform1i(gl::get_uniform_location(shader.id, sampler.as_str()), i as i32);
            // and finally bind the texture
            gl::bind_texture(GL_TEXTURE_2D, texture.id);
        }

        // draw mesh
        gl::bind_vertex_array(self.vao);
        gl::draw_elements_offset(GL_TRIANGLES, self.indices.len() as i32, GL_UNSIGNED_INT, 0);
        gl::bind_vertex_array(0);

        // always good practice to set everything back to defaults once configured.
        gl::active_texture(GL_TEXTURE0);
    }

    fn setup_mesh(&mut self) {
        // create buffers/arrays
        self.vao = gl::gen_vertex_array();
        self.vbo = gl::gen_buffer();
        self.ebo = gl::gen_buffer();

        gl::bind_vertex_array(self.vao);
        // load data into vertex buffers
        gl::bind_buffer(GL_ARRAY_BUFFER, self.vbo);
        // A great thing about structs with repr(C) is that their memory layout is sequential for all its items.
        // The effect is that we can simply pass a pointer to the struct and it translates perfectly to a glm::vec3/2 array which
        // again translates to 3/2 floats which translates to a byte array.
        gl::buffer_data(GL_ARRAY_BUFFER, self.vertices.as_slice(), GL_STATIC_DRAW);

        gl::bind_buffer(GL_ELEMENT_ARRAY_BUFFER, self.ebo);
        gl::buffer_data(GL_ELEMENT_ARRAY_BUFFER, self.indices.as_slice(), GL_STATIC_DRAW);

        // set the vertex attribute pointers
        let size = size_of::<Vertex>() as i32;
        // vertex Positions
        gl::enable_vertex_attrib_array(0);
        // gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, size, offset_of!(Vertex, Position));
        gl::vertex_attrib_pointer_offset(0, 3, GL_FLOAT, false, size, 0);
        // vertex normals
        gl::enable_vertex_attrib_array(1);
        // gl::vertex_attrib_pointer_offset(1, 3, GL_FLOAT, false, size, offset_of!(Vertex, Normal));
        gl::vertex_attrib_pointer_offset(1, 3, GL_FLOAT, false, size, 12); // 3 * 4
                                                                           // vertex texture coords
        gl::enable_vertex_attrib_array(2);
        // gl::vertex_attrib_pointer_offset(2, 2, GL_FLOAT, false, size, offset_of!(Vertex, TexCoords));
        gl::vertex_attrib_pointer_offset(2, 2, GL_FLOAT, false, size, 24); // 12 + 3 * 4
                                                                           // vertex tangent
        gl::enable_vertex_attrib_array(3);
        // gl::vertex_attrib_pointer_offset(3, 3, GL_FLOAT, false, size, offset_of!(Vertex, Tangent));
        gl::vertex_attrib_pointer_offset(3, 3, GL_FLOAT, false, size, 32); // 24 + 2 * 4
                                                                           // vertex bitangent
        gl::enable_vertex_attrib_array(4);
        // gl::vertex_attrib_pointer_offset(4, 3, GL_FLOAT, false, size, offset_of!(Vertex, Bitangent));
        gl::vertex_attrib_pointer_offset(4, 3, GL_FLOAT, false, size, 44); // 32 + 3 * 4

        gl::bind_vertex_array(0);
    }
}
