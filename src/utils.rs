use bevy::math::Vec2;

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn line_intersect(
    line_1_start: Vec2,
    line_1_end: Vec2,
    line_2_start: Vec2,
    line_2_end: Vec2,
) -> Option<Vec2> {
    let line_1 = line_1_end - line_1_start;
    let line_2 = line_2_end - line_2_start;
    let r_cross_s = cross_product(line_1, line_2);
    let a_to_c = line_2_start - line_1_start;
    let t = cross_product(a_to_c, line_2) / r_cross_s;
    let u = cross_product(a_to_c, line_1) / r_cross_s;

    if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
        Some(Vec2::new(
            line_1_start.x + t * line_1.x,
            line_1_start.y + t * line_1.y,
        ))
    } else {
        None
    }
}

pub fn cross_product(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}
