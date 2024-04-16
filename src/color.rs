use glam::DVec3 as Vec3;

use crate::Interval;

pub type Color = Vec3;

pub fn write_color(pixel_color: Color) {
    let intensity = Interval::new(0.000, 0.999);

    let r = linear_to_gamma(pixel_color.x);
    let g = linear_to_gamma(pixel_color.y);
    let b = linear_to_gamma(pixel_color.z);

    let r = (256.0 * intensity.clamp(r)) as i32;
    let g = (256.0 * intensity.clamp(g)) as i32;
    let b = (256.0 * intensity.clamp(b)) as i32;

    println!("{r} {g} {b}");
}

fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 {
        return linear_component.sqrt();
    }

    0.0
}
