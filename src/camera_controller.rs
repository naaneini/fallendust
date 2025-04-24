use ferrousgl::{GlWindow, WindowKey};
use glam::{Mat4, Vec3};

pub struct CameraController {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub sensitivity: f32,
}

impl CameraController {
    /// Creates a new `CameraController` with default values and a specified aspect ratio.
    pub fn new(aspect_ratio: f32) -> Self {
        CameraController {
            position: Vec3::new(0.0, 50.0, 5.0), // Default camera position
            target: Vec3::new(0.0, 0.0, 0.0),  // Default target (looking at the origin)
            up: Vec3::new(0.0, 1.0, 0.0),      // Default up vector
            fov: 45.0,                        // Field of view in degrees
            aspect_ratio,                     // Aspect ratio (width / height)
            near: 0.1,                        // Near clipping plane
            far: 1000.0,                       // Far clipping plane
            yaw: -90.0,                       // Default yaw
            pitch: 0.0,                       // Default pitch
            speed: 0.1,                       // Default movement speed
            sensitivity: 0.1,                 // Default mouse sensitivity
        }
    }

    /// Updates the camera's position based on input from the window.
    pub fn update(&mut self, window: &mut GlWindow) {
        // Calculate forward vector based on yaw and pitch
        let forward = Vec3::new(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        )
        .normalize();

        // Calculate right vector
        let right = forward.cross(self.up).normalize();

        // Movement speed
        let speed = self.speed;

        // Move left/right
        if window.is_key_held(WindowKey::A) {
            self.position -= right * speed;
        }
        if window.is_key_held(WindowKey::D) {
            self.position += right * speed;
        }

        // Move forward/backward
        if window.is_key_held(WindowKey::W) {
            self.position += forward * speed;
        }
        if window.is_key_held(WindowKey::S) {
            self.position -= forward * speed;
        }

        // Move up/down
        if window.is_key_held(WindowKey::Q) {
            self.position.y -= speed;
        }
        if window.is_key_held(WindowKey::E) {
            self.position.y += speed;
        }

        // Reset mouse
        let mouse_delta = window.get_mouse_delta();

        // Handle mouse input for rotation
        self.yaw += mouse_delta.0 as f32 * self.sensitivity;
        self.pitch -= mouse_delta.1 as f32 * self.sensitivity;

        // Clamp pitch to avoid gimbal lock
        self.pitch = self.pitch.clamp(-89.0, 89.0);

        // Update target based on new forward vector
        self.target = self.position + forward;

        // Update aspect ratio
        self.aspect_ratio = (window.get_window_size().0 as f32) / (window.get_window_size().1 as f32);

        // Handle mouse wrapping
        let mouse_pos = window.get_mouse_position();
        let window_pos = window.get_window_position();
        let window_size = window.get_window_size();

        let window_left = window_pos.0 as i32;
        let window_top = window_pos.1 as i32;
        let window_right = window_left + window_size.0 as i32;
        let window_bottom = window_top + window_size.1 as i32;

        if (mouse_pos.0 <= window_left.into()) {
            window.set_mouse_position((window_right - 1).into(), mouse_pos.1);
        } else if (mouse_pos.0 >= window_right.into()) {
            window.set_mouse_position((window_left + 1).into(), mouse_pos.1);
        }

        if (mouse_pos.1 <= window_top.into()) {
            window.set_mouse_position(mouse_pos.0, (window_bottom - 1).into());
        } else if (mouse_pos.1 >= window_bottom.into()) {
            window.set_mouse_position(mouse_pos.0, (window_top + 1).into());
        }
    }

    /// Generates the Model-View-Projection (MVP) matrix for the camera.
    pub fn get_vp(&self) -> Mat4 {
        let projection = Mat4::perspective_rh_gl(self.fov.to_radians(), self.aspect_ratio, self.near, self.far);
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        projection * view
    }
}