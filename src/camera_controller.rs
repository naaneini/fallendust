use glam::Vec3;

pub struct CameraController {
    pub position: Vec3,
}

impl CameraController {
    pub fn new() -> Self {
        CameraController {
            position: Vec3::new(0.0, 0.0, 0.0)
        }
    }
}