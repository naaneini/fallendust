use glam::IVec3;
use noise::{core::perlin, NoiseFn, Perlin};
use rayon::prelude::*; // Import rayon's parallel iterator traits

use super::scalar_data::ScalarData;

pub struct ScalarGenerator;

impl ScalarGenerator {
    pub fn generate(position: IVec3, seed: u32, mut chunk_size: u16) -> ScalarData {
        let perlin = Perlin::new(seed);
        let secondary_perlin = Perlin::new(seed.wrapping_add(1)); // Secondary noise layer
        let large_scale_perlin = Perlin::new(seed.wrapping_add(2)); // Large-scale 2D noise layer
        let fourth_perlin = Perlin::new(seed.wrapping_add(3)); // 4th noise layer

        chunk_size = chunk_size + 1;

        let dimensions = (
            chunk_size as usize + 2,
            chunk_size as usize + 2,
            chunk_size as usize + 2,
        );

        let grid_and_values: Vec<([f32; 3], f32)> = (0..dimensions.2)
            .into_par_iter() // Parallelize the outermost loop
            .flat_map_iter(|z| {
                (0..dimensions.1).flat_map(move |y| {
                    (0..dimensions.0).map(move |x| {
                        let world_x = position.x as f64 * (chunk_size - 1) as f64 + x as f64;
                        let world_y = position.y as f64 * (chunk_size - 1) as f64 + y as f64;
                        let world_z = position.z as f64 * (chunk_size - 1) as f64 + z as f64;

                        let grid_point = [world_x as f32, world_y as f32, world_z as f32];
                        
                        let small_details = perlin.get([world_x * 0.05, world_y * 0.05, world_z * 0.05]) as f32;
                        let small_detail_noise = secondary_perlin.get([world_x * 0.01, world_y * 0.000001, world_z * 0.01]) as f32;
                        let large_noise = large_scale_perlin.get([world_x * 0.005, world_y * 0.000001, world_z * 0.005]) as f32;

                        let value = ((large_noise*10.0) - (small_details*((0.5 + small_detail_noise) / 2.0))) - world_y as f32 * 0.1;

                        (grid_point, value)
                    })
                })
            })
            .collect();

        let (grid, values): (Vec<_>, Vec<_>) = grid_and_values.into_iter().unzip();

        ScalarData {
            grid,
            values,
            dimensions,
        }
    }
}