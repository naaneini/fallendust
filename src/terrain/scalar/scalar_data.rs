use glam::IVec3;

#[derive(Clone)]
pub struct ScalarData {
    pub grid: Vec<[f32; 3]>,       // 3D positions of the grid points
    pub values: Vec<f32>,          // Scalar values at each grid point
    pub dimensions: IVec3, // Dimensions of the scalar field (x, y, z)
}

impl ScalarData {
    /// Gets the value at specified grid coordinates
    /// Returns None if coordinates are out of bounds
    pub fn get_value(&self, coordinate: IVec3) -> Option<f32> {
        if coordinate.x >= self.dimensions.x || coordinate.y >= self.dimensions.y || coordinate.z >= self.dimensions.z {
            return None;
        }
        
        let index = (coordinate.z * self.dimensions.x * self.dimensions.y 
                  + coordinate.y * self.dimensions.x 
                  + coordinate.x) as usize;
        
        self.values.get(index).copied()
    }

    /// Sets the value at specified grid coordinates
    /// Returns Err if coordinates are out of bounds, Ok otherwise
    pub fn set_value(&mut self, coordinate: IVec3, value: f32) -> Result<(), String> {

        
        let index = (coordinate.z * self.dimensions.x * self.dimensions.y 
                  + coordinate.y * self.dimensions.x 
                  + coordinate.x) as usize;
        
        if let Some(v) = self.values.get_mut(index) {
            *v += value;
            Ok(())
        } else {
            Err("Index calculation error".to_string())
        }
    }
}