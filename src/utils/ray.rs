use glam::Vec3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// Create a new ray
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Get a point along the ray at a given distance
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    /// Check if the ray intersects with an axis-aligned bounding box (AABB)
    pub fn intersects_aabb(&self, aabb_min: Vec3, aabb_max: Vec3) -> bool {
        let inv_dir = self.direction.recip();
        let t1 = (aabb_min - self.origin) * inv_dir;
        let t2 = (aabb_max - self.origin) * inv_dir;

        let t_min = t1.min(t2);
        let t_max = t1.max(t2);

        let t_enter = t_min.max_element();
        let t_exit = t_max.min_element();

        t_enter <= t_exit && t_exit >= 0.0
    }
}