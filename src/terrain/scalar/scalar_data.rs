#[derive(Clone)]
pub struct ScalarData {
    pub grid: Vec<[f32; 3]>,       // 3D positions of the grid points
    pub values: Vec<f32>,          // Scalar values at each grid point
    pub dimensions: (usize, usize, usize), // Dimensions of the scalar field (x, y, z)
}

impl ScalarData {
    /// Get a mutable reference to the scalar value at a specific grid position
    pub fn get_mut(&mut self, position: glam::IVec3) -> Option<&mut f32> {
        // Ensure the position is within bounds
        
            // Calculate the 1D index from the 3D position
            let index = (position.z as usize) * (self.dimensions.0) * (self.dimensions.1) + (position.y as usize) * (self.dimensions.0) + (position.x as usize);

            // Return a mutable reference to the scalar value
            self.values.get_mut(index)
        
    }
}