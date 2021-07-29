use cgmath::{vec2, vec3};
use dx::gles::{core30::gl, enums::*};
use image::{DynamicImage::*, GenericImageView};
use std::path::{Path, PathBuf};
use tobj::LoadOptions;

use super::mesh::{Mesh, Texture, Vertex};
use super::shader::Shader;

#[derive(Default)]
pub struct Model {
    /*  Model Data */
    pub meshes: Vec<Mesh>,
    pub textures_loaded: Vec<Texture>, // stores all the textures loaded so far, optimization to make sure textures aren't loaded more than once.
    directory: String,
}

impl Model {
    /// constructor, expects a filepath to a 3D model.
    pub fn new<T>(path: T) -> Model
    where
        T: Into<PathBuf>,
    {
        let mut model = Model::default();
        model.load_model(path);
        model
    }

    pub fn draw(&self, shader: &Shader) {
        for mesh in &self.meshes {
            mesh.draw(shader);
        }
    }

    // loads a model from file and stores the resulting meshes in the meshes vector.
    fn load_model<T>(&mut self, path: T)
    where
        T: Into<PathBuf>,
    {
        let path: PathBuf = path.into();

        // retrieve the directory path of the filepath
        self.directory = path.parent().unwrap_or_else(|| Path::new("")).to_str().unwrap().into();
        let obj = tobj::load_obj(
            path,
            &LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        );

        let (models, materials) = obj.unwrap();
        for model in models {
            let mesh = &model.mesh;
            let num_vertices = mesh.positions.len() / 3;

            // data to fill
            let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);
            let indices: Vec<u32> = mesh.indices.clone();

            for idx in 0..num_vertices {
                vertices.push(Vertex {
                    position: vec3(mesh.positions[idx * 3], mesh.positions[idx * 3 + 1], mesh.positions[idx * 3 + 2]),
                    normal: vec3(mesh.normals[idx * 3], mesh.normals[idx * 3 + 1], mesh.normals[idx * 3 + 2]),
                    tex_coords: vec2(mesh.texcoords[idx * 2], mesh.texcoords[idx * 2 + 1]),
                    ..Vertex::default()
                })
            }

            // process material
            match &materials {
                Ok(materials) => {
                    let mut textures = Vec::new();
                    if let Some(material_id) = mesh.material_id {
                        let material = &materials[material_id];

                        // 1. diffuse map
                        if !material.diffuse_texture.is_empty() {
                            let texture = self.load_material_texture(&material.diffuse_texture, "texture_diffuse");
                            textures.push(texture);
                        }
                        // 2. specular map
                        if !material.specular_texture.is_empty() {
                            let texture = self.load_material_texture(&material.specular_texture, "texture_specular");
                            textures.push(texture);
                        }
                        // 3. normal map
                        if !material.normal_texture.is_empty() {
                            let texture = self.load_material_texture(&material.normal_texture, "texture_normal");
                            textures.push(texture);
                        }
                        // NOTE: no height maps
                    }

                    self.meshes.push(Mesh::new(vertices, indices, textures));
                }
                Err(err) => {
                    println!("{}", err)
                }
            }
        }
    }

    fn load_material_texture(&mut self, path: &str, type_name: &str) -> Texture {
        {
            let texture = self.textures_loaded.iter().find(|t| t.path == path);
            if let Some(texture) = texture {
                return texture.clone();
            }
        }

        let texture = Texture {
            id: texture_from_file(path, &self.directory),
            type_: type_name.into(),
            path: path.into(),
        };
        self.textures_loaded.push(texture.clone());
        texture
    }
}

fn texture_from_file(path: &str, directory: &str) -> u32 {
    let filename = format!("{}/{}", directory, path);

    let texture_id = gl::gen_texture();

    let img = image::open(&Path::new(&filename)).expect("Texture failed to load");
    let img = img.flipv();
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
