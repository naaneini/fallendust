use ferrousgl::Mesh;
use glam::IVec3;

use super::marching_cubes::marching_cubes_data_tables::MarchingCubesDataTables;
use super::scalar::scalar_generator::ScalarGenerator;
use super::marching_cubes::marching_cubes_generator::MarchingCubesGenerator;

pub struct TerrainChunk {
    pub position: IVec3,
    pub mesh: Mesh,
}

impl TerrainChunk {
    pub fn generate(
        position: IVec3,
        chunk_size: u16,
        seed: u32,
        data_tables: &MarchingCubesDataTables,
        isolevel: f32,
    ) -> Self {
        // Generate scalar data
        let scalar_data = ScalarGenerator::generate(position, seed, chunk_size);

        // Generate mesh data using marching cubes
        let (vertices, indices) = MarchingCubesGenerator::generate(data_tables.clone(), scalar_data, isolevel);

        // Create the mesh
        let mut mesh = Mesh::new();

        mesh.add_vertex_attributes(&[
            (0, 3, gl::FLOAT, false), // position
            (1, 3, gl::FLOAT, false), // normal
        ]);

        mesh.update_vertices(&vertices);
        mesh.update_indices(&indices);
        
        TerrainChunk { position, mesh }
    }

    pub fn get_mesh(&self) -> &Mesh {
        &self.mesh
    }
}