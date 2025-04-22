use std::collections::HashMap;
use ferrousgl::GlWindow;
use glam::UVec3;
use super::terrain_chunk::TerrainChunk;

pub struct TerrainManager {
    chunk_size: u16,
    chunks: HashMap<UVec3, TerrainChunk>,
}

impl TerrainManager {
    pub fn new() -> Self {
        TerrainManager {
            chunk_size: 64,
            chunks: HashMap::new(),  
        }
    }

    pub fn render(&mut self, window: &mut GlWindow) {
        for (position, chunk) in &self.chunks {
            window.render_mesh(chunk.get_mesh());
        }
    }
}

