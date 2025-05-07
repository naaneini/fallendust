use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::Path;
use std::time::Instant;
use ferrousgl::{texture, GlWindow, MipmapType, Shader, Texture};
use glam::{IVec3, Mat4, Vec3, Vec4};
use serde_json::de;
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
    last_chunk_position: IVec3,
    last_local_position: IVec3,
    chunks_to_remesh: HashSet<IVec3>,
    chunk_generation_start_time: Option<Instant>,
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
            texture.set_mipmap_type(MipmapType::Nearest);
        }

        TerrainManager {
            chunk_size: 64,
            chunks: HashMap::new(),
            data_tables,
            terrain_shader,
            textures,
            chunk_generation_queue: VecDeque::new(),
            seed: 0,
            isolevel: 0.5,
            last_chunk_position: IVec3::new(0, 0, 0),
            last_local_position: IVec3::new(0, 0, 0),
            chunks_to_remesh: HashSet::new(),
            chunk_generation_start_time: None,
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
        let mut lod_level = if distance < 2.0 {
            1 // Highest detail close to origin
        } else if distance < 5.0 {
            2
        } else if distance < 10.0 {
            4
        } else {
            8
        };
        lod_level = 1;
        
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

    pub fn clear_chunks(&mut self) {
        self.chunks.clear();
        self.chunk_generation_queue.clear();
    }

    pub fn force_generate_chunk(&mut self, position: IVec3) {
        if !self.chunks.contains_key(&position) {
            self.generate_chunk(position);
        }
    }

    pub fn enqueue_chunks_in_radius(&mut self, center: IVec3, render_distance: i32) {
        let mut positions_to_generate = Vec::new();
        self.chunk_generation_queue.clear(); // Clear the queue before adding new positions

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
        // Start the timer when the queue is populated
        self.chunk_generation_start_time = Some(Instant::now());
    }

    pub fn process_chunk_generation(&mut self) {
        if let Some(position) = self.chunk_generation_queue.pop_front() {
            self.generate_chunk(position);
        }

        // Check if the queue is empty and stop the timer
        if self.chunk_generation_queue.is_empty() {
            if let Some(start_time) = self.chunk_generation_start_time.take() {
                let duration = start_time.elapsed();
                println!("Total time elapsed generating all chunks: {:?}", duration);
            }
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
    
    pub fn create_sphere(&mut self, center: Vec3, radius: f32) {
        let radius_sq = radius * radius;
        let min = (center - Vec3::splat(radius)).floor();
        let max = (center + Vec3::splat(radius)).ceil();
    
        for x in (min.x as i32)..=(max.x as i32) {
            for y in (min.y as i32)..=(max.y as i32) {
                for z in (min.z as i32)..=(max.z as i32) {
                    let pos = Vec3::new(x as f32, y as f32, z as f32);
                    let offset = pos - center;
                    let distance_sq = offset.length_squared();
    
                    if distance_sq > radius_sq {
                        continue; // Skip points outside the sphere
                    }
    
                    let distance = distance_sq.sqrt();
                    let normalized_dist = distance / radius;
    
                    // Smooth cubic falloff (1.0 at center, 0.0 at radius)
                    let mut delta = 1.0 - 3.0 * normalized_dist.powi(2) + 2.0 * normalized_dist.powi(3);
    
                    // Clamp delta to ensure it's not negative
                    delta = delta.max(0.0);
    
                    self.place_voxel(pos, delta);
                }
            }
        }
        
        self. remesh_all_chunks();
    }

    pub fn place_voxel(&mut self, position: Vec3, delta: f32) {
        // Track which chunks need remeshing
        let mut chunks_to_remesh = HashSet::new();
    
        let chunk_size = self.chunk_size as usize;
    
        // Extract the values we need before any mutable borrows
        let mut last_chunk_position = self.last_chunk_position.clone();
        let mut last_local_position = self.last_local_position.clone();
    
        let target_position = position; // Use a mutable copy
    
        let mut local_position = IVec3::new(
            (position.z.rem_euclid(chunk_size as f32)) as i32,
            (position.y.rem_euclid(chunk_size as f32)) as i32,
            (position.x.rem_euclid(chunk_size as f32)) as i32,
        );

        let mut chunk_position = IVec3::new(
            (position.x / self.chunk_size as f32).floor() as i32,
            (position.y / self.chunk_size as f32).floor() as i32,
            (position.z / self.chunk_size as f32).floor() as i32,
        );

        if let Some(chunk) = self.chunks.get_mut(&chunk_position) {
            let is_different = chunk.position != last_chunk_position || last_local_position != local_position;
            if is_different {

            }
    
            last_chunk_position = chunk.position;
            last_local_position = local_position;
    
            chunk.modify_terrain(local_position, delta); // Full density value
            chunks_to_remesh.insert(chunk.position);
        }
    
        // Check if any coordinate is zero
       // println!("Local position:     {:?}", local_position);
        if local_position.z == 0 {
            chunk_position.x -= 1;
            local_position.z = 64 as i32;
        }
        if local_position.y == 0 {
            chunk_position.y -= 1;
            local_position.y = 64 as i32;
        }
        if local_position.x == 0 {
            chunk_position.z -= 1;
            local_position.x = 64 as i32;
        }

        if local_position.z == 1 {
            chunk_position.x -= 1;
            local_position.z = 65 as i32;
        }
        if local_position.y == 1 {
            chunk_position.y -= 1;
            local_position.y = 65 as i32;
        }
        if local_position.x == 1 {
            chunk_position.z -= 1;
            local_position.x = 65 as i32;
        }

        if local_position.z == 2 {
            chunk_position.x -= 1;
            local_position.z = 66 as i32;
        }
        if local_position.y == 2 {
            chunk_position.y -= 1;
            local_position.y = 66 as i32;
        }
        if local_position.x == 2 {
            chunk_position.z -= 1;
            local_position.x = 66 as i32;
        }
        //println!("Local position new: {:?}", local_position);
        //println!("Target position:    {:?}", target_position);
    
        if let Some(chunk) = self.chunks.get_mut(&chunk_position) {
            let is_different = chunk.position != last_chunk_position || last_local_position != local_position;
            if is_different {

            }
    
            last_chunk_position = chunk.position;
            last_local_position = local_position;
    
            chunk.modify_terrain(local_position, delta); // Full density value
            chunks_to_remesh.insert(chunk.position);
        }
        self.last_chunk_position = last_chunk_position;
        self.last_local_position = last_local_position;
    
        // Remesh affected chunks
        for chunk_pos in chunks_to_remesh {
            self.chunks_to_remesh.insert(chunk_pos);
        }
    }

    pub fn remesh_all_chunks(&mut self) {
        let chunks_to_remesh = self.chunks_to_remesh.clone();
        self.chunks_to_remesh.clear();
    
        for chunk_pos in chunks_to_remesh {
            if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
                chunk.remesh_chunk(&self.data_tables, self.isolevel, 1);
            }
        }
    }

    pub fn e (&mut self) {
        self.get_active_chunks_count();

        self.last_chunk_position = IVec3::new(0, 0, 0);
    }

    pub fn new_modify_terrain(&mut self, position: IVec3, delta: f32) {
        if let Some(chunk) = self.get_chunk_for_voxel(position.as_vec3()) {
            chunk.modify_terrain(position, delta);
            //chunk.remesh_chunk(&self.data_tables, self.isolevel, 1);
        }
    }

    pub fn place_voxel_in_chunk(&mut self, chunk_position: IVec3, local_position: IVec3, density_delta: f32) {
        if let Some(chunk) = self.chunks.get_mut(&chunk_position) {
            chunk.modify_terrain(local_position, density_delta);
            chunk.remesh_chunk(&self.data_tables, self.isolevel, 1);
        } else {
            println!("Chunk at position {:?} does not exist.", chunk_position);
        }
    }

    pub fn raycast(&mut self, ray: &Ray, max_distance: f32) -> Option<Vec3> {
        let mut current_distance = 0.0;
        let step_size = 0.01; // Adjust for desired precision

        while current_distance < max_distance {
            let current_position = ray.at(current_distance);
            
            // Get the chunk for the current voxel position
            let chunk_position = IVec3::new(
                (current_position.x / self.chunk_size as f32).floor() as i32,
                (current_position.y / self.chunk_size as f32).floor() as i32,
                (current_position.z / self.chunk_size as f32).floor() as i32,
            );

            if let Some(chunk) = self.chunks.get_mut(&chunk_position) {
                // Convert world position to local position within the chunk
                let local_position = IVec3::new(
                    (current_position.x.rem_euclid(self.chunk_size as f32)) as i32,
                    (current_position.y.rem_euclid(self.chunk_size as f32)) as i32,
                    (current_position.z.rem_euclid(self.chunk_size as f32)) as i32,
                );

                // Check if the local position is within the chunk bounds
                if local_position.x >= 0 && local_position.x < self.chunk_size as i32 &&
                   local_position.y >= 0 && local_position.y < self.chunk_size as i32 &&
                   local_position.z >= 0 && local_position.z < self.chunk_size as i32 {
                    
                    // Get the density value at the local position
                    let density = chunk.scalar_data.get_value(local_position);

                    // Check if the density is below the threshold
                    if density > Some(0.5) {
                        return Some(current_position);
                    }
                }
            } else {
                // If the chunk doesn't exist, continue the raycast without generating
                // This avoids generating chunks just for raycasting
            }

            current_distance += step_size;
        }

        None // No voxel found within the max_distance
    }
}

