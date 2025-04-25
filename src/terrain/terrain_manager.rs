use std::collections::HashMap;
use std::collections::VecDeque;
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
    chunk_generation_queue: VecDeque<IVec3>, // Queue for chunk positions to generate
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
            chunk_size: 64,
            chunks: HashMap::new(),
            data_tables,
            terrain_shader,
            textures,
            chunk_generation_queue: VecDeque::new(),
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

    pub fn enqueue_chunks_in_radius(&mut self, center: IVec3, render_distance: i32) {
        let mut positions_to_generate = Vec::new();

        for x in -render_distance as i32..=render_distance as i32 {
            for y in -render_distance as i32..=render_distance as i32 {
                for z in -render_distance as i32..=render_distance as i32 {
                    let offset = IVec3::new(x, y, z);
                    let position = center + offset;

                    // Check if the chunk is already generated
                    if !self.chunks.contains_key(&position) {
                        positions_to_generate.push((offset.length_squared(), position));
                    }
                }
            }
        }

        // Sort positions by distance from the center (center-out generation)
        positions_to_generate.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Enqueue positions for generation
        for (_, position) in positions_to_generate {
            self.chunk_generation_queue.push_back(position);
        }
    }

    pub fn process_chunk_generation(&mut self, seed: u32, isolevel: f32) {
        if let Some(position) = self.chunk_generation_queue.pop_front() {
            self.generate_chunk(position, seed, isolevel);
        }
    }

    pub fn render(&mut self, window: &mut GlWindow, vp: Mat4) {
        self.textures[0].bind(0); // Bind the first texture to texture unit 0
        self.textures[1].bind(1);

        self.terrain_shader.bind_program();
        self.terrain_shader.set_uniform_3f("uLightDir", -0.8, 1.0, 0.5);
        self.terrain_shader.set_uniform_texture("uGrassTex", 0);
        self.terrain_shader.set_uniform_texture("uStoneTex", 1);

        //window.set_rendering_type(ferrousgl::RenderingType::Wireframe);

        for (_, chunk) in &self.chunks {
            let model = Mat4::from_translation(Vec3::new(chunk.position.z as f32 * self.chunk_size as f32, chunk.position.y as f32 * self.chunk_size as f32, chunk.position.x as f32 * self.chunk_size as f32));
            let mvp = vp * model;
            self.terrain_shader.set_uniform_matrix_4fv("uMVP", &mvp.to_cols_array());

            window.render_mesh(chunk.get_mesh());
        }

        self.terrain_shader.unbind_program();
    }
}

