use crate::{
    color::{self, Color},
    random_vec3_on_unit_disc, random_vec3_unit, vec3, Hittable, Interval, Point3, Ray, Vec3,
};

pub struct Camera {
    image_width: i32,
    samples_per_pixel: i32,
    pixel_samples_scale: f64,
    max_depth: i32,
    image_height: i32,

    /// Variation angle of rays through each pixel
    defocus_angle: f64,

    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,

    // Defocus disk radius
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

pub struct CameraParams {
    pub aspect_ratio: f64,
    pub image_width: i32,
    pub samples_per_pixel: i32,
    pub max_depth: i32,
    pub vfov: f64,
    pub lookfrom: Point3,
    pub lookat: Point3,
    pub vup: Vec3,
    pub defocus_angle: f64,
    pub focus_dist: f64,
}

impl Camera {
    pub fn new(params: CameraParams) -> Self {
        let CameraParams {
            aspect_ratio,
            image_width,
            defocus_angle,
            focus_dist,
            lookat,
            lookfrom,
            max_depth,
            vfov,
            samples_per_pixel,
            vup,
            ..
        } = params;

        let image_height = (image_width as f64 / aspect_ratio) as i32;

        let center = lookfrom;

        // Determine viewport dimensions.
        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focus_dist;
        let viewport_width = viewport_height * ((image_width as f64) / (image_height as f64));

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        let w = (lookfrom - lookat).normalize();
        let u = (vup.cross(w)).normalize();
        let v = w.cross(u);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = viewport_width * u; // Vector across viewport horizontal edge
        let viewport_v = viewport_height * -v; // Vector down viewport vertical edge

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left = center - (focus_dist * w) - viewport_u / 2.0 - viewport_v / 2.0;

        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        // Calculate the camera defocus disk basis vectors.
        let defocus_radius = focus_dist * ((defocus_angle / 2.0).to_radians()).tan();

        Self {
            image_width,
            image_height,
            samples_per_pixel,
            max_depth,
            pixel_samples_scale: 1.0 / samples_per_pixel as f64,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            defocus_angle,
            defocus_disk_u: u * defocus_radius,
            defocus_disk_v: v * defocus_radius,
        }
    }

    pub fn render(&mut self, world: impl Hittable) {
        println!("P3");
        println!("{} {}", self.image_width, self.image_height);
        println!("255");

        for j in 0..self.image_height {
            eprintln!("Scanlines remaining: {}", self.image_height - j);
            for i in 0..self.image_width {
                let mut pixel_color = Color::default();
                for _ in 0..self.samples_per_pixel {
                    let r = self.get_ray(i, j);
                    pixel_color += Self::ray_color(&r, self.max_depth, &world);
                }

                color::write_color(self.pixel_samples_scale * pixel_color);
            }
        }
    }

    fn get_ray(&self, i: i32, j: i32) -> Ray {
        // Construct a camera ray originating from the defocus disk and directed at a randomly
        // sampled point around the pixel location i, j.

        let offset = Self::sample_square();
        let pixel_sample = self.pixel00_loc
            + (i as f64 + offset.x) * self.pixel_delta_u
            + (j as f64 + offset.y) * self.pixel_delta_v;

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_sample - ray_origin;

        Ray::new(ray_origin, ray_direction)
    }

    fn ray_color(r: &Ray, depth: i32, world: &impl Hittable) -> Color {
        if depth <= 0 {
            return Color::ZERO;
        }

        if let Some(rec) = world.hit(r, Interval::new(0.001, f64::INFINITY)) {
            let mat = rec.mat.as_ref();

            if let Some((scattered, attenuation)) = mat.scatter(r, &rec) {
                return attenuation * Self::ray_color(&scattered, depth - 1, world);
            }
            let direction = rec.normal + random_vec3_unit();
            return 0.5 * Self::ray_color(&Ray::new(rec.p, direction), depth - 1, world);
        }

        let unit_direction = r.direction.normalize();
        let a = 0.5 * (unit_direction.y + 1.0);

        (1.0 - a) * Color::splat(1.) + a * Color::new(0.5, 0.7, 1.0)
    }

    /// Returns the vector to a random point in the [-.5,-.5]-[+.5,+.5] unit square.
    fn sample_square() -> Vec3 {
        vec3(
            rand::random::<f64>() - 0.5,
            rand::random::<f64>() - 0.5,
            0.0,
        )
    }

    fn defocus_disk_sample(&self) -> Vec3 {
        let p = random_vec3_on_unit_disc();

        self.center + p.x * self.defocus_disk_u + p.y * self.defocus_disk_v
    }
}
