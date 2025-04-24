use std::collections::HashMap;
use std::path::Path;
use ferrousgl::{texture, GlWindow, MipmapType, Shader, Texture};
use glam::{IVec3, Mat4, Vec3, Vec4};

use super::terrain_chunk::TerrainChunk;
use super::marching_cubes::marching_cubes_data_tables::MarchingCubesDataTables;

pub struct TerrainManager {
    chunk_size: u16,
    chunks: HashMap<IVec3, TerrainChunk>,
    data_tables: MarchingCubesDataTables,
    terrain_shader: Shader,
    textures: Vec<Texture>, // List of textures
}

impl TerrainManager {
    pub fn new() -> Self {
        let data_tables = MarchingCubesDataTables::load_from_files("./assets/data/marching_cubes_tables/").unwrap();

        let terrain_shader = Shader::new_from_file(
            Path::new("./assets/shaders/terrain/vertex.glsl"),
            Path::new("./assets/shaders/terrain/fragment.glsl"),
        ).unwrap();

        let textures = vec![
            Texture::new_from_file(Path::new("./assets/media/textures/grass.jpg")).unwrap(),
            Texture::new_from_file(Path::new("./assets/media/textures/stone.jpg")).unwrap(),
        ];
        for texture in &textures {
            texture.bind(0);
            texture.set_mipmap_type(MipmapType::Linear);
        }

        TerrainManager {
            chunk_size: 128,
            chunks: HashMap::new(),
            data_tables,
            terrain_shader,
            textures,
        }
    }

    pub fn generate_chunk(&mut self, position: IVec3, seed: u32, isolevel: f32) {
        let chunk = TerrainChunk::generate(
            position,
            self.chunk_size,
            seed,
            &self.data_tables,
            isolevel,
        );

        self.chunks.insert(position, chunk);
    }

    pub fn render(&mut self, window: &mut GlWindow, vp: Mat4) {
        self.textures[0].bind(0); // Bind the first texture to texture unit 0
        self.textures[1].bind(1);

        self.terrain_shader.bind_program();
        self.terrain_shader.set_uniform_3f("uLightDir", -0.8, 1.0, 0.5);
        self.terrain_shader.set_uniform_texture("uGrassTex", 0);
        self.terrain_shader.set_uniform_texture("uStoneTex", 1);

        for (_, chunk) in &self.chunks {
            let model = Mat4::from_translation(Vec3::new(chunk.position.z as f32 * self.chunk_size as f32, chunk.position.y as f32 * self.chunk_size as f32, chunk.position.x as f32 * self.chunk_size as f32));
            let mvp = vp * model;
            self.terrain_shader.set_uniform_matrix_4fv("uMVP", &mvp.to_cols_array());

            window.render_mesh(chunk.get_mesh());
        }

        self.terrain_shader.unbind_program();
    }
}

