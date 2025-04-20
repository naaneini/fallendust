use std::collections::HashMap;
use glam::UVec3;
use super::terrain_chunk::TerrainChunk;

pub struct TerrainManager {
    chunk_size: u16,
    chunks: HashMap<UVec3, TerrainChunk>,
}

impl TerrainManager {
    pub fn new() -> Self {
        TerrainManager {
            64,
            HashMap<UVec3, TerrainChunk>::new(),
        }
    }

    pub fn render(&mut dyn window: GlWindow) {
        window.render_mesh();
    }
}

