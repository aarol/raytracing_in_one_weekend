use std::{ops::Range, rc::Rc};

use camera::{Camera, CameraParams};
use color::Color;
use glam::{dvec3 as vec3, DVec3 as Vec3};
use material::{Dielectric, Lambertian, Material, Metal};
use rand::Rng;

mod camera;
mod color;
mod material;

fn main() {
    let mut world = HittableList::new();

    let mat_ground = Rc::new(Lambertian {
        albedo: Color::new(0.5, 0.5, 0.5),
    });
    world.add(Box::new(Sphere {
        center: vec3(0.0, -1000.0, -1.0),
        radius: 1000.0,
        mat: mat_ground,
    }));

    let mut rng = rand::thread_rng();
    let mut random = || rng.gen::<f64>();

    for a in -11..11 {
        for b in -11..11 {
            let center = Point3::new(a as f64 + 0.9 * random(), 0.2, b as f64 + 0.9 * random());

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let num = random();
                let mat: Rc<dyn Material> = if num < 0.8 {
                    let albedo = random_vec3() * random_vec3();

                    Rc::new(Lambertian { albedo })
                } else if num < 0.95 {
                    let albedo = Color::new(0.5, 1.0, random());
                    let fuzz = rand::thread_rng().gen_range((0.0)..0.5);
                    Rc::new(Metal { albedo, fuzz })
                } else {
                    Rc::new(Dielectric {
                        refraction_index: 1.5,
                    })
                };

                world.add(Box::new(Sphere {
                    center,
                    radius: 0.2,
                    mat,
                }));
            }
        }
    }

    let mat1 = Rc::new(Dielectric {
        refraction_index: 1.5,
    });
    world.add(Box::new(Sphere {
        center: vec3(0.0, 1.0, 0.0),
        radius: 1.0,
        mat: mat1,
    }));

    let mat2 = Rc::new(Lambertian {
        albedo: Color::new(0.4, 0.2, 0.1),
    });
    world.add(Box::new(Sphere {
        center: vec3(-4.0, 1.0, 0.0),
        radius: 1.0,
        mat: mat2,
    }));

    let mat3 = Rc::new(Metal {
        albedo: Color::new(0.7, 0.6, 0.5),
        fuzz: 0.0,
    });
    world.add(Box::new(Sphere {
        center: vec3(4.0, 1.0, 0.0),
        radius: 1.0,
        mat: mat3,
    }));

    let mut cam = Camera::new(CameraParams {
        aspect_ratio: 16.0 / 9.0,
        image_width: 600,
        samples_per_pixel: 100,
        max_depth: 25,
        vfov: 20.0,
        lookfrom: Point3::new(13.0, 2.0, 3.0),
        lookat: Point3::ZERO,
        vup: vec3(0., 1., 0.),
        defocus_angle: 0.6,
        focus_dist: 10.0,
    });

    cam.render(world);
}

type Point3 = Vec3;

#[derive(Default)]
struct Ray {
    origin: Point3,
    direction: Vec3,
}

impl Ray {
    pub fn new(orig: Point3, dir: Vec3) -> Self {
        Self {
            origin: orig,
            direction: dir,
        }
    }

    fn at(&self, t: f64) -> Point3 {
        self.origin + t * self.direction
    }
}

struct HitRecord {
    p: Point3,
    normal: Vec3,
    mat: Rc<dyn Material>,
    t: f64,
    front_face: bool,
}

impl HitRecord {
    fn new(p: Point3, t: f64, mat: Rc<dyn Material>, r: &Ray, outward_normal: Vec3) -> Self {
        // Sets the hit record normal vector.
        // NOTE: the parameter `outward_normal` is assumed to have unit length.
        let front_face = r.direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        Self {
            p,
            normal,
            mat,
            t,
            front_face,
        }
    }
}

trait Hittable {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord>;
}

struct Sphere {
    center: Point3,
    radius: f64,
    mat: Rc<dyn Material>,
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let oc = self.center - r.origin;

        let a = r.direction.length_squared();
        let h = r.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = h * h - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        let mut root = (h - sqrtd) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return None;
            }
        }

        let p = r.at(root);

        let rec = HitRecord::new(
            p,
            root,
            self.mat.clone(),
            r,
            (p - self.center) / self.radius,
        );

        Some(rec)
    }
}

struct HittableList {
    objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    fn new() -> Self {
        Self { objects: vec![] }
    }

    fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object)
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let mut hit_anything = None;
        let mut closest_so_far = ray_t.max;

        for obj in &self.objects {
            if let Some(rec) = obj.hit(r, Interval::new(ray_t.min, closest_so_far)) {
                closest_so_far = rec.t;
                hit_anything = Some(rec);
            }
        }

        hit_anything
    }
}

#[derive(Clone, Copy, Default)]
struct Interval {
    min: f64,
    max: f64,
}

impl Interval {
    fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    fn surrounds(&self, x: f64) -> bool {
        self.min < x && x < self.max
    }

    fn clamp(&self, x: f64) -> f64 {
        if x < self.min {
            return self.min;
        }
        if x > self.max {
            return self.max;
        }

        x
    }
}

pub fn random_vec3() -> Vec3 {
    vec3(rand::random(), rand::random(), rand::random())
}

pub fn random_vec3_range(r: Range<f64>) -> Vec3 {
    let mut rng = rand::thread_rng();
    vec3(
        rng.gen_range(r.clone()),
        rng.gen_range(r.clone()),
        rng.gen_range(r),
    )
}

pub fn random_vec3_unit() -> Vec3 {
    loop {
        let p = random_vec3_range((-1.0)..1.0);
        if p.length_squared() < 1.0 {
            return p.normalize();
        }
    }
}

pub fn random_vec3_on_hempishere(normal: &Vec3) -> Vec3 {
    let on_unit_sphere = random_vec3_unit();
    if normal.dot(on_unit_sphere) > 0.0 {
        // In the same hemisphere as the normal
        on_unit_sphere
    } else {
        -on_unit_sphere
    }
}
pub fn random_vec3_on_unit_disc() -> Vec3 {
    let mut rng = rand::thread_rng();

    loop {
        let p = vec3(rng.gen_range((-1.0)..1.0), rng.gen_range((-1.0)..1.0), 0.0);
        if p.length_squared() < 1.0 {
            return p;
        }
    }
}
