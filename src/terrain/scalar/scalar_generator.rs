use glam::IVec3;
use noise::{core::perlin, NoiseFn, Perlin};

use super::scalar_data::ScalarData;

pub struct ScalarGenerator;

impl ScalarGenerator {
    pub fn generate(position: IVec3, seed: u32, mut chunk_size: u16) -> ScalarData {
        let perlin = Perlin::new(seed);
        let secondary_perlin = Perlin::new(seed.wrapping_add(1)); // Secondary noise layer
        let large_scale_perlin = Perlin::new(seed.wrapping_add(2)); // Large-scale 2D noise layer

        let mut grid = Vec::new();
        let mut values = Vec::new();

        chunk_size = chunk_size + 1;

        let dimensions = (
            chunk_size as usize + 2,
            chunk_size as usize + 2,
            chunk_size as usize + 2,
        );

        for z in 0..dimensions.2 {
            for y in 0..dimensions.1 {
                for x in 0..dimensions.0 {
                    let world_x = position.x as f64 * (chunk_size - 1) as f64 + x as f64;
                    let world_y = position.y as f64 * (chunk_size - 1) as f64 + y as f64;
                    let world_z = position.z as f64 * (chunk_size - 1) as f64 + z as f64;

                    let hills = perlin.get([world_x * 0.1, world_y * 0.1, world_z * 0.1]);
                    let mountains = secondary_perlin.get([world_x * 0.05, world_z * 0.05]);

                    let final_value = hills as f32 - ((0.3 - (mountains as f32 * 0.25)) * world_y as f32);

                    grid.push([world_x as f32, world_y as f32, world_z as f32]);
                    values.push(final_value);
                }
            }
        }

        ScalarData {
            grid,
            values,
            dimensions,
        }
    }
}