use xenofrost::core::math::Vec3;

pub(crate) struct Ray {
    origin: Vec3,
    direction: Vec3
}

impl Ray {
    pub(crate) fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction
        }
    }

    pub(crate) fn get_origin(&self) -> Vec3 {
        self.origin
    }

    pub(crate) fn get_direction(&self) -> Vec3 {
        self.direction
    }
}