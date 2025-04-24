pub struct ScalarData {
    pub grid: Vec<[f32; 3]>,       // 3D positions of the grid points
    pub values: Vec<f32>,          // Scalar values at each grid point
    pub dimensions: (usize, usize, usize), // Dimensions of the scalar field (x, y, z)
}