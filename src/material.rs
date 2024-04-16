use glam::DVec3 as Vec3;
use rand::random;

use crate::{color::Color, random_vec3_unit, HitRecord, Ray};

pub trait Material {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)>;
}

pub struct Lambertian {
    pub albedo: Color,
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let mut scatter_direction = rec.normal + random_vec3_unit();

        if vec3_near_zero(&scatter_direction) {
            scatter_direction = rec.normal;
        }

        let scattered = Ray::new(rec.p, scatter_direction);
        let attenuation = self.albedo;

        Some((scattered, attenuation))
    }
}

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let mut reflected = vec3_reflect(r_in.direction, rec.normal);
        reflected = reflected.normalize() + self.fuzz.min(1.0) * random_vec3_unit();
        let scattered = Ray::new(rec.p, reflected);
        let attenuation = self.albedo;

        Some((scattered, attenuation))
    }
}

pub struct Dielectric {
    /// Refractive index in vacuum or air, or the ratio of the material's refractive index over
    /// the refractive index of the enclosing media
    pub refraction_index: f64,
}

impl Dielectric {
    fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
        // Use Schlick's approximation for reflectance.
        let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        r0 = r0 * r0;

        r0 + (1.0 - r0) * f64::powi(1.0 - cosine, 5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        let ri = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = r_in.direction.normalize();
        let cos_theta = f64::min(rec.normal.dot(-unit_direction), 1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;

        let direction = if cannot_refract || Self::reflectance(cos_theta, ri) > random() {
            vec3_reflect(unit_direction, rec.normal)
        } else {
            vec3_refract(unit_direction, rec.normal, ri)
        };

        let attenuation = Color::ONE;
        let scattered = Ray::new(rec.p, direction);

        Some((scattered, attenuation))
    }
}

pub fn vec3_reflect(vec: Vec3, n: Vec3) -> Vec3 {
    vec - 2.0 * vec.dot(n) * n
}

pub fn vec3_refract(uv: Vec3, n: Vec3, etai_over_etat: f64) -> Vec3 {
    let cos_theta = f64::min(n.dot(-uv), 1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -((1.0 - r_out_perp.length_squared()).abs()).sqrt() * n;

    r_out_perp + r_out_parallel
}

pub fn vec3_near_zero(vec: &Vec3) -> bool {
    let s = 1.0e-8;

    vec.x.abs() < s && vec.y.abs() < s && vec.z.abs() < s
}
