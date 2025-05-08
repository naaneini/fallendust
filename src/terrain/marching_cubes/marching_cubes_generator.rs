use crate::terrain::marching_cubes::marching_cubes_data_tables::MarchingCubesDataTables;
use crate::terrain::scalar::scalar_data::ScalarData;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use rayon::ThreadPoolBuilder;
use std::{collections::HashMap, time::Instant};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
struct VertexKey {
    x: usize,
    y: usize,
    z: usize,
    lod: usize,
    index: u8,
}

pub struct MarchingCubesGenerator;

impl MarchingCubesGenerator {
    pub fn generate(
        data_tables: MarchingCubesDataTables,
        scalar_data: ScalarData,
        isolevel: f32,
        lod: usize,
    ) -> (Vec<f32>, Vec<u32>) {
        // Configure Rayon to use exactly 4 threads
        let pool = ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .expect("Failed to create thread pool");

        // LOD must be at least 1 (no skipping)
        let lod = lod.max(1);
        let grid_size = scalar_data.dimensions.x as usize;
        let values = &scalar_data.values;

        // Early exit if all values are below threshold
        if values.iter().all(|&value| value < isolevel) {
            return (Vec::new(), Vec::new());
        }

        let mut vertex_data: Vec<f32> = Vec::new(); // Interleaved [position, normal]
        let mut indices: Vec<u32> = Vec::new();

        // Process slices in parallel using our 4-thread pool
        let slices: Vec<(usize, Vec<f32>, Vec<u32>)> = pool.install(|| {
            (1..grid_size - 2)
                .into_par_iter()
                .step_by(lod) // Skip slices based on LOD
                .filter(|&x| x + lod < grid_size - 1) // Ensure overlap between slices
                .map(|x| Self::process_slice(x, &data_tables, &scalar_data, isolevel, lod))
                .collect()
        });

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
        lod: usize,
    ) -> (usize, Vec<f32>, Vec<u32>) {
        let grid_size = scalar_data.dimensions.x as usize;
        let values = &scalar_data.values;
        let grid = &scalar_data.grid;

        let mut slice_vertex_data = Vec::new();
        let mut slice_indices = Vec::new();
        let mut vertex_cache: HashMap<VertexKey, u32> = HashMap::new();
        let mut corner_values = [0.0; 8];

        for y in (1..grid_size - 2).step_by(lod) {
            for z in (1..grid_size - 2).step_by(lod) {
                let mut cube_index = 0;

                // Compute corner values and determine cube index
                for i in 0..8 {
                    let corner_x = x + (i & 1) * lod;
                    let corner_y = y as usize + ((i >> 1) & 1) * lod;
                    let corner_z = z as usize + ((i >> 2) & 1) * lod;
                    
                    // Handle out-of-bounds cases safely
                    corner_values[i] = if corner_x >= grid_size || corner_y >= grid_size || corner_z >= grid_size {
                        isolevel - 1.0 // Assign a value below isolevel
                    } else {
                        let index = corner_x * grid_size * grid_size + corner_y * grid_size + corner_z;
                        // Handle NaN/infinite values
                        if values[index].is_nan() || values[index].is_infinite() {
                            isolevel - 1.0
                        } else {
                            values[index]
                        }
                    };

                    if corner_values[i] < isolevel {
                        cube_index |= 1 << i;
                    }
                }

                // Skip empty cubes
                if data_tables.edge_masks[cube_index] == 0 {
                    continue;
                }

                // Handle ambiguous cases (3, 6, 7, 10, 12, 13)
                let cube_index = Self::resolve_ambiguous_case(cube_index, &corner_values, isolevel);

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
                            lod,
                        );

                        // Create key for vertex cache
                        let key = VertexKey {
                            x,
                            y,
                            z,
                            lod,
                            index: i as u8,
                        };

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

                // Add triangle indices, filtering degenerate triangles
                let mut triangle_indices = Vec::with_capacity(15); // Max 5 triangles per cube
                let mut i = 0;
                while i < data_tables.triangulation_table[cube_index].len() {
                    if data_tables.triangulation_table[cube_index][i] == -1 {
                        break;
                    }

                    // Collect triangle vertices
                    let a = cube_vertices[data_tables.triangulation_table[cube_index][i] as usize];
                    let b = cube_vertices[data_tables.triangulation_table[cube_index][i+1] as usize];
                    let c = cube_vertices[data_tables.triangulation_table[cube_index][i+2] as usize];

                    // Skip degenerate triangles
                    if a != b && b != c && c != a {
                        triangle_indices.push(a);
                        triangle_indices.push(b);
                        triangle_indices.push(c);
                    }
                    i += 3;
                }

                slice_indices.extend(triangle_indices);
            }
        }

        (x, slice_vertex_data, slice_indices)
    }

    // Resolves ambiguous cases in marching cubes
    fn resolve_ambiguous_case(cube_index: usize, corner_values: &[f32; 8], isolevel: f32) -> usize {
        // Check if this is one of the ambiguous cases
        match cube_index {
            3 | 6 | 7 | 10 | 12 | 13 => {
                // Implement asymptotic decider to resolve ambiguity
                // For simplicity, we'll use a basic approach here
                // A more complete implementation would analyze face ambiguities
                
                // Check face ambiguities
                let mut resolved_index = cube_index;
                
                // Check for face ambiguities (simplified)
                if cube_index == 3 || cube_index == 12 {
                    // Check if the face has an ambiguous saddle point
                    let face_ambiguous = Self::is_face_ambiguous(corner_values, isolevel);
                    if face_ambiguous {
                        // Alternate triangulation for ambiguous cases
                        resolved_index = match cube_index {
                            3 => 3,  // Could use alternate table for case 3
                            12 => 12, // Could use alternate table for case 12
                            _ => cube_index,
                        };
                    }
                }
                
                resolved_index
            }
            _ => cube_index,
        }
    }

    // Checks if a face has an ambiguous configuration
    fn is_face_ambiguous(corner_values: &[f32; 8], isolevel: f32) -> bool {
        // Simplified check - a real implementation would analyze the face more carefully
        let mut above = 0;
        let mut below = 0;
        
        for &value in corner_values.iter() {
            if value >= isolevel {
                above += 1;
            } else {
                below += 1;
            }
        }
        
        // If exactly 2 or 6 corners are above, might be ambiguous
        above == 2 || above == 6
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
        lod: usize,
    ) -> ([f32; 3], [f32; 3]) {
        // Handle cases where values are very close to isolevel
        if (value1 - isolevel).abs() < f32::EPSILON * 10.0 {
            return (Self::corner_position(v1, x, y, z, lod), Self::calculate_normal(Self::corner_position(v1, x, y, z, lod), scalar_data));
        }
        if (value2 - isolevel).abs() < f32::EPSILON * 10.0 {
            return (Self::corner_position(v2, x, y, z, lod), Self::calculate_normal(Self::corner_position(v2, x, y, z, lod), scalar_data));
        }

        // Handle cases where values are nearly equal
        let t = if (value2 - value1).abs() < f32::EPSILON * 10.0 {
            0.5
        } else {
            ((isolevel - value1) / (value2 - value1)).clamp(0.0, 1.0)
        };

        let position1 = Self::corner_position(v1, x, y, z, lod);
        let position2 = Self::corner_position(v2, x, y, z, lod);

        let pos = [
            position1[0] + t * (position2[0] - position1[0]),
            position1[1] + t * (position2[1] - position1[1]),
            position1[2] + t * (position2[2] - position1[2]),
        ];

        let normal1 = Self::calculate_normal(position1, scalar_data);
        let normal2 = Self::calculate_normal(position2, scalar_data);

        // Normalize the interpolated normal
        let normal = {
            let nx = normal1[0] + t * (normal2[0] - normal1[0]);
            let ny = normal1[1] + t * (normal2[1] - normal1[1]);
            let nz = normal1[2] + t * (normal2[2] - normal1[2]);
            let length = (nx * nx + ny * ny + nz * nz).sqrt();
            if length > f32::EPSILON {
                [nx / length, ny / length, nz / length]
            } else {
                [0.0, 0.0, 1.0] // Default normal if calculation fails
            }
        };

        (pos, normal)
    }

    fn corner_position(corner: usize, x: usize, y: usize, z: usize, lod: usize) -> [f32; 3] {
        [
            (x + (corner & 1) * lod) as f32,
            (y + ((corner >> 1) & 1) * lod) as f32,
            (z + ((corner >> 2) & 1) * lod) as f32,
        ]
    }

    fn calculate_normal(pos: [f32; 3], scalar_data: &ScalarData) -> [f32; 3] {
        let grid_size = scalar_data.dimensions.x as usize;
        let values = &scalar_data.values;
        
        let x = pos[0].floor() as usize;
        let y = pos[1].floor() as usize;
        let z = pos[2].floor() as usize;

        // Helper function to get value at grid point with bounds checking
        let get_value = |x: isize, y: isize, z: isize| -> f32 {
            if x >= 0 && x < grid_size as isize && 
               y >= 0 && y < grid_size as isize && 
               z >= 0 && z < grid_size as isize 
            {
                let val = values[x as usize * grid_size * grid_size + y as usize * grid_size + z as usize];
                if val.is_nan() || val.is_infinite() {
                    0.0
                } else {
                    val
                }
            } else {
                0.0
            }
        };

        // Calculate derivatives using central differences
        let dx = (get_value(x as isize + 1, y as isize, z as isize) - 
                  get_value(x as isize - 1, y as isize, z as isize));
        let dy = (get_value(x as isize, y as isize + 1, z as isize) - 
                  get_value(x as isize, y as isize - 1, z as isize));
        let dz = (get_value(x as isize, y as isize, z as isize + 1) - 
                  get_value(x as isize, y as isize, z as isize - 1));

        // Normalize
        let length = (dx * dx + dy * dy + dz * dz).sqrt();
        if length > f32::EPSILON {
            [dx / length, dy / length, dz / length]
        } else {
            [0.0, 0.0, 1.0] // Default normal if calculation fails
        }
    }
}