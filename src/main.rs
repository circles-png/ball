use std::{
    io::{stdin, BufRead},
    thread::sleep,
    time::Duration,
};

use glam::{vec3, Quat, Vec3};
use itertools::Itertools;

const SIZE: f32 = 130.;
const BALL_RADIUS: f32 = 10.;
const CAMERA_Y: f32 = 50.;
const PLANE_OFFSET: f32 = -150.;
const GROUND_Z: f32 = -10.;

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
fn main() {
    const SCALE: &str = " `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@";
    let mut light = vec3(70., 50., 100.);
    loop {
        let brightnesses = (0..SIZE as usize)
            .step_by(2)
            .rev()
            .map(|y| {
                (0..SIZE as usize)
                    .map(move |x| {
                        let x = x as f32;
                        let y = y as f32;
                        let camera = Vec3::Y * CAMERA_Y;
                        let view_point = vec3(
                            (x - SIZE / 2.) * 0.9,
                            CAMERA_Y + PLANE_OFFSET,
                            y - SIZE / 2.,
                        );
                        let ray = (view_point - camera).normalize();
                        let int = sphere_line_int(camera, ray);
                        let Some(closest) = int.closest(camera) else {
                            let Some(int) = plane_line_int(camera, ray) else {
                                return f32::NEG_INFINITY;
                            };
                            if int.y > CAMERA_Y {
                                return f32::NEG_INFINITY;
                            }
                            let to_light_direction = (light - int).normalize();
                            let normal = Vec3::Z;
                            let reflected = (((to_light_direction + int)
                                + 2. * ((int + normal) - (int + to_light_direction)))
                                - int)
                                .normalize();
                            let blocked = matches!(
                                sphere_line_int(int, to_light_direction),
                                SphereLineIntResult::Tangent(_) | SphereLineIntResult::Through(_)
                            );
                            if blocked {
                                return -7.;
                            }
                            return reflected.dot(-ray.normalize())
                                / (light - int).length_squared()
                                * 100_000.;
                        };
                        let to_light_direction = (light - closest).normalize();
                        let normal =
                            closest.normalize() * to_light_direction.dot(closest.normalize());
                        let reflected = (((to_light_direction + closest)
                            + 2. * ((closest + normal) - (closest + to_light_direction)))
                            - closest)
                            .normalize();
                        reflected.dot(-ray.normalize()) / (light - closest).length_squared()
                            * 100_000.
                    })
                    .collect_vec()
            })
            .collect_vec();
        println!(
            "\x1b[2J{}",
            brightnesses
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|brightness| {
                            SCALE
                                .chars()
                                .nth(
                                    brightness
                                        .mul_add(2., 18.)
                                        .min(SCALE.len() as f32 - 1.)
                                        .max(0.) as usize,
                                )
                                .unwrap()
                        })
                        .collect::<String>()
                })
                .join("\n")
        );
        light = Quat::from_axis_angle(Vec3::Z, 0.05) * light;
        sleep(Duration::from_secs_f32(1. / 60.));
    }
}

enum SphereLineIntResult {
    None,
    Tangent(Vec3),
    Through([Vec3; 2]),
}

impl SphereLineIntResult {
    fn closest(self, camera: Vec3) -> Option<Vec3> {
        match self {
            Self::None => None,
            Self::Tangent(p) => Some(p),
            Self::Through([p1, p2]) => {
                if (p1 - camera).length_squared() < (p2 - camera).length_squared() {
                    Some(p1)
                } else {
                    Some(p2)
                }
            }
        }
    }
}

fn sphere_line_int(line_point: Vec3, line_direction: Vec3) -> SphereLineIntResult {
    let nabla = (line_direction.dot(line_point)).mul_add(
        line_direction.dot(line_point),
        -BALL_RADIUS.mul_add(-BALL_RADIUS, line_point.length_squared()),
    );
    match nabla {
        ..0. => SphereLineIntResult::None,
        0. => SphereLineIntResult::Tangent(
            -(line_direction.dot(line_point)) * line_direction + line_point,
        ),
        _ => SphereLineIntResult::Through([
            (-(line_direction.dot(line_point)) - nabla.sqrt()) * line_direction + line_point,
            (-(line_direction.dot(line_point)) + nabla.sqrt()) * line_direction + line_point,
        ]),
    }
}

fn plane_line_int(line_point: Vec3, line_direction: Vec3) -> Option<Vec3> {
    let d = ((Vec3::Z * GROUND_Z - line_point).dot(Vec3::Z)) / (line_direction.dot(Vec3::Z));
    Some(line_point + line_direction * d).filter(|int| int.is_finite())
}
