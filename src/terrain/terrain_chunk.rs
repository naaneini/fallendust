use ferrousgl::Mesh;
use glam::UVec3;

pub struct TerrainChunk {
    pub position: UVec3,
    pub data: ScalarData,
    pub mesh: Mesh,
}

impl TerrainChunk {
    pub fn generate(position: UVec3, chunk_size: u16) -> Self {
        let scalar_data = ScalarGenerator::generate(position, seed, chunk_size);

        let mesh_data = MarchingCubesGeneator::generate_marching_cubes(scalar_data);

        let mesh = Mesh::new();

        TerrainChunk {
            position,
            data,
            mesh,
        }
    }

    pub fn get_mesh(&self) {
        return self.mesh;
    }
}