use xenofrost::core::math::Vec3;

pub(crate) const NITRORAY_FLOAT_EPSILON: f32 = 0.0001;

pub(crate) fn get_direction_vector_from_yaw_and_pitch(yaw: f32, pitch: f32) -> Vec3 {
    let x = f32::sin(yaw.to_radians()) * f32::cos(pitch.to_radians());
    let y = f32::sin(pitch.to_radians());
    let z = f32::cos(yaw.to_radians()) * f32::cos(pitch.to_radians());
    Vec3::new(x, y, z).normalize()
}

pub(crate) fn are_floats_equal(f1: f32, f2: f32) -> bool {
    (f1 - f2).abs() < NITRORAY_FLOAT_EPSILON
}