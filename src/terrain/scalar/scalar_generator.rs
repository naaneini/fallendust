use glam::IVec3;
use noise::{core::value, NoiseFn, Perlin};
use rayon::prelude::*;
use std::time::Instant;
use super::scalar_data::ScalarData;

pub struct ScalarGenerator;

impl ScalarGenerator {
    pub fn generate(position: IVec3, seed: u32, chunk_size: u16) -> ScalarData {        
        // Precompute Perlin noise instances
        let perlin = Perlin::new(seed);
        let perlin2 = Perlin::new(seed.wrapping_add(1)); // Different seeds for variety
        let perlin3 = Perlin::new(seed.wrapping_add(2));
        let perlin4 = Perlin::new(seed.wrapping_add(3));

        let dimensions = IVec3::new(chunk_size as i32 +3, chunk_size as i32 +3, chunk_size as i32 +3);

        // Precompute position offsets
        let base_x = position.x as f64 * (chunk_size as f64);
        let base_y = position.y as f64 * (chunk_size as f64);
        let base_z = position.z as f64 * (chunk_size as f64);

        let (grid, values): (Vec<_>, Vec<_>) = (0..dimensions.x) // Swap x and z loops
            .into_par_iter()
            .flat_map_iter(|x| { // Process x first
                let world_x = base_x + x as f64;
                (0..dimensions.z).flat_map(move |y| {
                    let world_y = base_y + y as f64;
                    (0..dimensions.y).map(move |z| { // Process z last
                        let world_z = base_z + z as f64;
                        
                        // Grid point
                        let grid_point = [world_x as f32, world_y as f32, world_z as f32];

                        let noise1 = perlin.get([world_x * 0.01, world_y * 0.01, world_z * 0.01]) as f32;
                        let noise2 = perlin2.get([world_x * 0.05, world_y * 0.05, world_z * 0.05]) as f32 * 0.5; // Different frequency and amplitude
                        let noise3 = perlin3.get([world_x * 0.1, world_y * 0.1, world_z * 0.1]) as f32 * 0.001; // Even higher frequency, lower amplitude
                        let noise4 = perlin4.get([world_x * 0.2, world_y * 0.2, world_z * 0.2]) as f32 * 0.075; // Even higher frequency, lower amplitude
                        
                        let value = noise1 + noise2 + noise3 + noise4 - ((world_y) * 0.025) as f32;
                        

                        (grid_point, value)
                    })
                })
            })
            .unzip();

        ScalarData {
            grid,
            values,
            dimensions,
        }
    }
}