use std::panic::AssertUnwindSafe;

use ferrousgl::Mesh;
use glam::{IVec3, Vec3};

use super::marching_cubes::marching_cubes_data_tables::MarchingCubesDataTables;
use super::marching_cubes::marching_cubes_generator::MarchingCubesGenerator;
use super::scalar::scalar_data::ScalarData;
use super::scalar::scalar_generator::ScalarGenerator;
use crate::utils::ray::Ray;

pub struct TerrainChunk {
    pub position: IVec3,
    pub mesh: Mesh,
    pub is_empty: bool,
    pub scalar_data: ScalarData,
}

impl TerrainChunk {
    pub fn generate(
        position: IVec3,
        chunk_size: u16,
        seed: u32,
        data_tables: &MarchingCubesDataTables,
        isolevel: f32,
        lod: usize,
    ) -> Self {
        // Generate scalar data
        let scalar_data = ScalarGenerator::generate(position, seed, chunk_size);

        // Generate mesh data using marching cubes
        let (vertices, indices) =
            MarchingCubesGenerator::generate(data_tables.clone(), scalar_data.clone(), isolevel, lod);

        let mut is_empty = false;
        if vertices.is_empty() {
            is_empty = true;
        }

        // Create the mesh
        let mut mesh = Mesh::new();

        mesh.add_vertex_attributes(&[
            (0, 3, gl::FLOAT, false), // position
            (1, 3, gl::FLOAT, false), // normal
        ]);

        mesh.update_vertices(&vertices);
        mesh.update_indices(&indices);

        TerrainChunk {
            position,
            mesh,
            is_empty,
            scalar_data,
        }
    }

    pub fn get_mesh(&self) -> &Mesh {
        &self.mesh
    }

    /// Modify the scalar data at a specific position and remesh
    pub fn modify_terrain(
        &mut self,
        local_position: IVec3,
        delta: f32,
    ) {
        self.scalar_data.set_value(local_position, delta);
    }

    pub fn remesh_chunk(
        &mut self,
        data_tables: &MarchingCubesDataTables,
        isolevel: f32,
        lod: usize
    ) {
        // Regenerate the mesh
        let (vertices, indices) =
            MarchingCubesGenerator::generate(data_tables.clone(), self.scalar_data.clone(), isolevel, lod);

        self.is_empty = vertices.is_empty();

        self.mesh.update_vertices(&vertices);
        self.mesh.update_indices(&indices);

        self.mesh.update_vertices(&vertices);
        self.mesh.update_indices(&indices);
    }
}
