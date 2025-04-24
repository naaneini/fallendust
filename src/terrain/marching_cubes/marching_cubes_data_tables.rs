use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct MarchingCubesDataTables {
    pub edge_masks: Vec<u32>,
    pub edge_vertex_indices: Vec<[usize; 2]>,
    pub triangulation_table: Vec<Vec<i32>>,
}

impl MarchingCubesDataTables {
    pub fn load_from_files<P: AsRef<Path>>(base_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let edge_masks_path = base_path.as_ref().join("edge_masks.json");
        let edge_vertex_indices_path = base_path.as_ref().join("edge_vertex_indices.json");
        let triangulation_table_path = base_path.as_ref().join("triangulation_table.json");

        let edge_masks: Vec<u32> = Self::load_json_file(&edge_masks_path)?;
        let edge_vertex_indices: Vec<[usize; 2]> = Self::load_json_file(&edge_vertex_indices_path)?;
        let triangulation_table: Vec<Vec<i32>> = Self::load_json_file(&triangulation_table_path)?;

        Ok(Self {
            edge_masks,
            edge_vertex_indices,
            triangulation_table,
        })
    }

    fn load_json_file<T: for<'de> Deserialize<'de>, P: AsRef<Path>>(path: P) -> Result<T, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data = serde_json::from_reader(reader)?;
        Ok(data)
    }
}