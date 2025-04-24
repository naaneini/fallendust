use crate::terrain::marching_cubes::marching_cubes_data_tables::MarchingCubesDataTables;
use crate::terrain::scalar::scalar_data::ScalarData;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq)]
struct VertexKey(usize, usize, usize, u8);

pub struct MarchingCubesGenerator;

impl MarchingCubesGenerator {
    pub fn generate(
        data_tables: MarchingCubesDataTables,
        scalar_data: ScalarData,
        isolevel: f32,
    ) -> (Vec<f32>, Vec<u32>) {
        let grid_size = scalar_data.dimensions.0;
        let values = &scalar_data.values;

        // Early exit if all values are below threshold
        if values.iter().all(|&value| value < isolevel) {
            return (Vec::new(), Vec::new());
        }

        let mut vertex_data: Vec<f32> = Vec::new(); // Interleaved [position, normal]
        let mut indices: Vec<u32> = Vec::new();

        // Process slices in parallel
        let slices: Vec<(usize, Vec<f32>, Vec<u32>)> = (1..grid_size - 2)
            .into_par_iter()
            .map(|x| Self::process_slice(x, &data_tables, &scalar_data, isolevel))
            .collect();

        // Merge results in order
        let mut vertex_offset = 0;
        for (_x, mut slice_vertex_data, mut slice_indices) in slices {
            // Adjust indices to global vertex list
            for index in &mut slice_indices {
                *index += vertex_offset;
            }

            indices.append(&mut slice_indices);
            vertex_data.append(&mut slice_vertex_data);

            vertex_offset = (vertex_data.len() / 6) as u32; // 6 floats per vertex (position + normal)
        }

        (vertex_data, indices)
    }

    fn process_slice(
        x: usize,
        data_tables: &MarchingCubesDataTables,
        scalar_data: &ScalarData,
        isolevel: f32,
    ) -> (usize, Vec<f32>, Vec<u32>) {
        let grid_size = scalar_data.dimensions.0;
        let values = &scalar_data.values;
        let grid = &scalar_data.grid;

        let mut slice_vertex_data = Vec::new();
        let mut slice_indices = Vec::new();
        let mut vertex_cache = HashMap::new();
        let mut corner_values = [0.0; 8];

        for y in 1..grid_size - 2 {
            for z in 1..grid_size - 2 {
                let mut cube_index = 0;

                // Compute corner values and determine cube index
                for i in 0..8 {
                    let corner_x = x + (i & 1);
                    let corner_y = y + ((i >> 1) & 1);
                    let corner_z = z + ((i >> 2) & 1);
                    let index = corner_x * grid_size * grid_size + corner_y * grid_size + corner_z;
                    corner_values[i] = values[index];
                    if corner_values[i] < isolevel {
                        cube_index |= 1 << i;
                    }
                }

                // Skip empty cubes
                if data_tables.edge_masks[cube_index] == 0 {
                    continue;
                }

                // Process edges
                let mut cube_vertices = [0u32; 12];
                for i in 0..12 {
                    if (data_tables.edge_masks[cube_index] & (1 << i)) != 0 {
                        let [v1, v2] = data_tables.edge_vertex_indices[i];
                        let (pos, normal) = Self::interpolate_vertex(
                            x, y, z, 
                            v1 as usize, 
                            v2 as usize, 
                            corner_values[v1 as usize], 
                            corner_values[v2 as usize],
                            isolevel,
                            scalar_data,
                        );

                        // Create key for vertex cache
                        let key = VertexKey(x, y, z, i as u8);

                        // Check if vertex already exists
                        cube_vertices[i] = match vertex_cache.get(&key) {
                            Some(&index) => index,
                            None => {
                                let index = (slice_vertex_data.len() / 6) as u32;
                                slice_vertex_data.extend_from_slice(&pos);
                                slice_vertex_data.extend_from_slice(&normal);
                                vertex_cache.insert(key, index);
                                index
                            }
                        };
                    }
                }

                // Add triangle indices
                for &triangle in data_tables.triangulation_table[cube_index].iter() {
                    if triangle == -1 {
                        break;
                    }
                    slice_indices.push(cube_vertices[triangle as usize]);
                }
            }
        }

        (x, slice_vertex_data, slice_indices)
    }

    fn interpolate_vertex(
        x: usize,
        y: usize,
        z: usize,
        v1: usize,
        v2: usize,
        value1: f32,
        value2: f32,
        isolevel: f32,
        scalar_data: &ScalarData,
    ) -> ([f32; 3], [f32; 3]) {
        let t = (isolevel - value1) / (value2 - value1);

        let position1 = Self::corner_position(v1, x, y, z);
        let position2 = Self::corner_position(v2, x, y, z);

        let pos = [
            position1[0] + t * (position2[0] - position1[0]),
            position1[1] + t * (position2[1] - position1[1]),
            position1[2] + t * (position2[2] - position1[2]),
        ];

        let normal1 = Self::calculate_normal(position1, scalar_data);
        let normal2 = Self::calculate_normal(position2, scalar_data);

        let normal = [
            normal1[0] + t * (normal2[0] - normal1[0]),
            normal1[1] + t * (normal2[1] - normal1[1]),
            normal1[2] + t * (normal2[2] - normal1[2]),
        ];

        (pos, normal)
    }

    fn corner_position(corner: usize, x: usize, y: usize, z: usize) -> [f32; 3] {
        [
            (x + (corner & 1)) as f32,
            (y + ((corner >> 1) & 1)) as f32,
            (z + ((corner >> 2) & 1)) as f32,
        ]
    }

    fn calculate_normal(pos: [f32; 3], scalar_data: &ScalarData) -> [f32; 3] {
        let grid_size = scalar_data.dimensions.0;
        let values = &scalar_data.values;
        
        let x = pos[0].floor() as usize;
        let y = pos[1].floor() as usize;
        let z = pos[2].floor() as usize;

        // Helper function to get value at grid point
        let get_value = |x, y, z| {
            values[x * grid_size * grid_size + y * grid_size + z]
        };

        // Calculate derivatives using central differences
        let dx = if x > 0 && x < grid_size - 1 {
            get_value(x + 1, y, z) - get_value(x - 1, y, z)
        } else {
            0.0
        };

        let dy = if y > 0 && y < grid_size - 1 {
            get_value(x, y + 1, z) - get_value(x, y - 1, z)
        } else {
            0.0
        };

        let dz = if z > 0 && z < grid_size - 1 {
            get_value(x, y, z + 1) - get_value(x, y, z - 1)
        } else {
            0.0
        };

        // Normalize
        let length = (dx * dx + dy * dy + dz * dz).sqrt();
        if length != 0.0 {
            [dx / length, dy / length, dz / length]
        } else {
            [0.0, 0.0, 0.0]
        }
    }
}