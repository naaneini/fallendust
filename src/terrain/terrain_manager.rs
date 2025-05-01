use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::Path;
use ferrousgl::{texture, GlWindow, MipmapType, Shader, Texture};
use glam::{IVec3, Mat4, Vec3, Vec4};
use crate::utils::ray::Ray; // Ensure Ray is imported

use super::terrain_chunk::TerrainChunk;
use super::marching_cubes::marching_cubes_data_tables::MarchingCubesDataTables;

pub struct TerrainManager {
    pub chunk_size: u16,
    pub chunks: HashMap<IVec3, TerrainChunk>,
    data_tables: MarchingCubesDataTables,
    pub terrain_shader: Shader,
    pub textures: Vec<Texture>, // List of textures
    chunk_generation_queue: VecDeque<IVec3>, // Queue for chunk positions to generate
    seed: u32,
    isolevel: f32,
}

impl TerrainManager {
    pub fn new() -> Self {
        let data_tables = MarchingCubesDataTables::load_from_files("./assets/data/marching_cubes_tables/").unwrap();

        let terrain_shader = Shader::new_from_file(
            Path::new("./assets/shaders/terrain/vertex.glsl"),
            Path::new("./assets/shaders/terrain/fragment.glsl"),
        ).unwrap();

        let textures = vec![
            Texture::new_from_file(Path::new("./assets/media/textures/grass.png")).unwrap(),
            Texture::new_from_file(Path::new("./assets/media/textures/grass_normal.png")).unwrap(),
            Texture::new_from_file(Path::new("./assets/media/textures/rock.png")).unwrap(),
            Texture::new_from_file(Path::new("./assets/media/textures/rock_normal.png")).unwrap(),
        ];
        for texture in &textures {
            texture.bind(0);
            texture.set_mipmap_type(MipmapType::Linear);
        }

        TerrainManager {
            chunk_size: 32,
            chunks: HashMap::new(),
            data_tables,
            terrain_shader,
            textures,
            chunk_generation_queue: VecDeque::new(),
            seed: 0,
            isolevel: 0.5,
        }
    }

    pub fn get_active_chunks_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn get_empty_active_chunks_count(&self) -> usize {
        self.chunks.values().filter(|chunk| chunk.is_empty).count()
    }

    pub fn generate_chunk(&mut self, position: IVec3) {
        // Calculate distance from origin (0,0,0)
        let distance = position.as_vec3().length();
        
        // Determine LOD level based on distance
        // You can adjust these thresholds as needed
        let lod_level = if distance < 2.0 {
            1 // Highest detail close to origin
        } else if distance < 5.0 {
            2
        } else if distance < 10.0 {
            4
        } else {
            8
        };
        
        let chunk = TerrainChunk::generate(
            position,
            self.chunk_size,
            self.seed,
            &self.data_tables,
            self.isolevel,
            lod_level
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
        positions_to_generate.sort_by(|a: &(i32, IVec3), b| a.0.partial_cmp(&b.0).unwrap());

        // Enqueue positions for generation
        for (_, position) in positions_to_generate {
            self.chunk_generation_queue.push_back(position);
        }
    }

    pub fn process_chunk_generation(&mut self) {
        if let Some(position) = self.chunk_generation_queue.pop_front() {
            self.generate_chunk(position);
        }
    }

    pub fn render(&mut self, window: &mut GlWindow, vp: Mat4) {
        self.textures[0].bind(0); // Bind the first texture to texture unit 0
        self.textures[1].bind(1);
        self.textures[2].bind(2);
        self.textures[3].bind(3);
        self.textures[4].bind(4);
        self.textures[5].bind(5);

        self.terrain_shader.bind_program();
        self.terrain_shader.set_uniform_3f("uLightDir", -0.8, 1.0, 0.5);
        self.terrain_shader.set_uniform_texture("uGrassTex", 0);
        self.terrain_shader.set_uniform_texture("uGrassNormal", 1);
        self.terrain_shader.set_uniform_texture("uRockTex", 2);
        self.terrain_shader.set_uniform_texture("uRockNormal",3);

        for (_, chunk) in &self.chunks {
            if chunk.is_empty {
                continue;
            }

            let model = Mat4::from_translation(
                Vec3::new(
                    chunk.position.x as f32 * self.chunk_size as f32,
                    chunk.position.y as f32 * self.chunk_size as f32,
                    chunk.position.z as f32 * self.chunk_size as f32
                ));
            let mvp = vp * model;
            self.terrain_shader.set_uniform_matrix_4fv("uMVP", &mvp.to_cols_array());

            window.render_mesh(chunk.get_mesh());
        }

        self.terrain_shader.unbind_program();
    }

    pub fn get_chunk_for_voxel(&mut self, pos: Vec3) -> Option<&mut TerrainChunk> {
        // Calculate the chunk position
        let chunk_position = IVec3::new(
            (pos.x / self.chunk_size as f32).floor() as i32,
            (pos.y / self.chunk_size as f32).floor() as i32,
            (pos.z / self.chunk_size as f32).floor() as i32,
        );
    
        // Ensure the chunk exists, generate it if necessary
        if !self.chunks.contains_key(&chunk_position) {
            self.generate_chunk(chunk_position);
        }
    
        // Return mutable reference to the chunk
        self.chunks.get_mut(&chunk_position)
    }
    
    pub fn place_sphere(&mut self, center: Vec3, radius: f32) {
        // Track which chunks need remeshing
        let mut chunks_to_remesh = HashSet::new();
        
        // Calculate the bounding box of the sphere
        let min_x = (center.x - radius).floor() as i32;
        let max_x = (center.x + radius).ceil() as i32;
        let min_y = (center.y - radius).floor() as i32;
        let max_y = (center.y + radius).ceil() as i32;
        let min_z = (center.z - radius).floor() as i32;
        let max_z = (center.z + radius).ceil() as i32;
    
        let chunk_size = self.chunk_size as f32;
        
        // First pass: modify all voxels with smooth values
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let pos = Vec3::new(x as f32, y as f32, z as f32);
                    let distance = pos.distance(center);
                    
                    // Calculate a smooth value based on distance from sphere surface
                    // This creates a smooth transition at the edges
                    let value = if distance < radius - 1.0 {
                        // Inside the sphere (with 1 unit margin)
                        1.0
                    } else if distance > radius + 1.0 {
                        // Outside the sphere (with 1 unit margin)
                        0.0
                    } else {
                        // In the transition zone (1 unit thick)
                        // Smoothly interpolate between 1 and 0
                        0.5 - (distance - radius) * 0.5
                    };
    
                    if value > 0.0 {
                        if let Some(chunk) = self.get_chunk_for_voxel(pos) {
                            let local_position = IVec3::new(
                                (pos.z.rem_euclid(chunk_size as f32)) as i32,
                                (pos.y.rem_euclid(chunk_size as f32)) as i32,
                                (pos.x.rem_euclid(chunk_size as f32)) as i32,
                            );
                            chunk.modify_terrain(local_position, value);
                            chunks_to_remesh.insert(chunk.position);
                        }
                    }
                }
            }
        }
        
        // Second pass: remesh affected chunks
        for chunk_pos in chunks_to_remesh {
            if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
                chunk.remesh_chunk(&self.data_tables, self.isolevel, 1);
            }
        }
    }

    pub fn modify_terrain_with_raycast(&mut self, ray: Ray, max_distance: f32, radius: f32, add: bool) {
        let mut distance = 0.0;
        let step = 0.1; // Step size for ray traversal
        let chunk_size = self.chunk_size as f32;
        let isolevel = self.isolevel; // Use the isolevel from the terrain manager

        let mut previous_value = None;

        while distance < max_distance {
            let point = ray.at(distance);

            if let Some(chunk) = self.get_chunk_for_voxel(point) {
                let local_position = IVec3::new(
                    (point.x.rem_euclid(chunk_size)) as i32,
                    (point.y.rem_euclid(chunk_size)) as i32,
                    (point.z.rem_euclid(chunk_size)) as i32,
                );

                if let Some(current_value) = chunk.scalar_data.get_mut(local_position) {
                    if let Some(prev_value) = previous_value {
                        // Check if we crossed the isolevel
                        if (prev_value < isolevel && *current_value >= isolevel)
                            || (prev_value >= isolevel && *current_value < isolevel)
                        {
                            // Place the modification at the border
                            let delta = if add { 1.0 } else { -1.0 };
                            chunk.modify_terrain(local_position, delta);

                            // Optionally, apply the modification to a radius
                            if radius > 0.0 {
                                self.place_sphere(point, radius);
                            }

                            return; // Stop after finding the border
                        }
                    }

                    previous_value = Some(*current_value);
                }
            }

            distance += step;
        }
    }
}

