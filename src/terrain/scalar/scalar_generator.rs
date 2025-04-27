use glam::IVec3;
use noise::{core::perlin, NoiseFn, Perlin};
use rayon::prelude::*;
use serde::de; // Import rayon's parallel iterator traits

use super::scalar_data::ScalarData;

pub struct ScalarGenerator;

impl ScalarGenerator {
    pub fn generate(position: IVec3, seed: u32, mut chunk_size: u16) -> ScalarData {
        let perlin = Perlin::new(seed);
        let perlin2 = Perlin::new(seed);
        let perlin3 = Perlin::new(seed);
        let perlin4 = Perlin::new(seed);

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
                        
                        let noise1 = perlin.get([world_x * 0.01, world_y * 0.00001, world_z * 0.01]);
                        let noise2 = perlin.get([world_x * 0.03, world_y * 0.03, world_z * 0.03]);
                        let mut detail_noise = perlin2.get([world_x * 0.1, world_y * 0.1, world_z * 0.1]);
                        detail_noise = (detail_noise * 0.5) + 0.5;
                        let mut detail_noise2 = perlin3.get([world_x * 0.01, world_y * 0.001, world_z * 0.01]);
                        detail_noise2 = (detail_noise2 * 0.5) + 0.5;

                        let value = ((noise2 as f32 + noise1 as f32 * 5.0) + ((detail_noise * detail_noise2) as f32 * 1.25)) - world_y as f32 * 0.1;
                        //(noise2 as f32 + noise1 as f32 * 5.0) + 

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