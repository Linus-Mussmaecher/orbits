use coffee::graphics::{self, Frame, Point, Vector, Window, WindowSettings};
use coffee::input::keyboard;
use coffee::load::Task;
use coffee::{Game, Result, Timer};

fn main() -> Result<()> {
    OrbitsInstance::run(WindowSettings {
        title: String::from("Orbits"),
        size: (1280, 1024),
        resizable: true,
        fullscreen: false,
        maximized: false,
    })
}

struct OrbitsInstance {
    objects: Vec<SpaceObject>,
}

impl OrbitsInstance {
    const GRAVITY: f32 = 1.0;
    const ROT_ACCELERATION: f32 = 0.1;
    const LIN_ACCELARATION: f32 = 0.02;
}

impl Game for OrbitsInstance {
    type Input = coffee::input::Keyboard;
    type LoadingScreen = coffee::load::loading_screen::ProgressBar;

    fn load(_window: &Window) -> Task<OrbitsInstance> {
        coffee::load::Join::join((
            graphics::Image::load("./assets/sun.png"),
            graphics::Image::load("./assets/ship.png"),
        ))
        .map(|(sun, ship)| OrbitsInstance {
            objects: vec![
                // Ship
                SpaceObject {
                    position: Point::from([256.0, 0.0]),
                    velocity: Vector::from([0.0, 2.0]),
                    angle: 0.0,
                    mass: 1.0,
                    sprite: ship,
                    size: 16.0,
                },
                // Sun
                SpaceObject {
                    position: Point::from([0.0, 0.0]),
                    velocity: Vector::zeros(),
                    angle: 0.0,
                    mass: 1024.0,
                    sprite: sun,
                    size: 96.0,
                },
            ],
        })
    }

    fn interact(&mut self, input: &mut Self::Input, window: &mut Window) {
        // Screen interaction
        if input.is_key_pressed(keyboard::KeyCode::F11) {
            window.toggle_fullscreen();
        }

        if let Some(ship) = self.objects.first_mut() {
            // Accelerate when W is pressed
            if input.is_key_pressed(keyboard::KeyCode::W) {
                ship.velocity += nalgebra::Rotation2::new(ship.angle)
                    .transform_vector(&Vector::x())
                    * Self::LIN_ACCELARATION;
            }
            // Turn with A/D
            if input.is_key_pressed(keyboard::KeyCode::A) {
                ship.angle += Self::ROT_ACCELERATION;
            }
            if input.is_key_pressed(keyboard::KeyCode::D) {
                ship.angle -= Self::ROT_ACCELERATION;
            }
        }
    }

    fn update(&mut self, _window: &Window) {
        // For every object, calculate the gravitational influence of all other objects on it.
        let accelerations = self
            .objects
            .iter()
            .map(|object| {
                let mut a = Vector::zeros();

                for attractor in self.objects.iter() {
                    let dist = attractor.position - object.position;
                    if dist.norm() != 0.0 {
                        a += dist.normalize() * Self::GRAVITY * object.mass * attractor.mass
                            / dist.norm_squared();
                    }
                }

                a
            })
            .collect::<Vec<_>>();

        // Then apply accelerations and velocities.
        for (object, accleration) in self.objects.iter_mut().zip(accelerations.iter()) {
            object.velocity += accleration / object.mass;
            object.position += object.velocity;
        }
    }

    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        // Clear the current frame
        frame.clear(graphics::Color::BLACK);

        // Get weighted average position
        let mut total_mass = 0.0;
        let mut center = Vector::zeros();

        for object in self.objects.iter() {
            center -= object.mass * object.position.to_homogeneous().xy();
            total_mass += object.mass;
        }

        center /= total_mass;

        center += Vector::from([frame.width() / 2., frame.height() / 2.]);

        let mut min_scale: f32 = 1.0;

        let (w, h) = (frame.width(), frame.height());

        for object in self.objects.iter() {
            let w_scale = (object.position.x - center.x).abs() / w * 5.0;
            let h_scale = (object.position.y - center.y).abs() / h * 5.0;

            dbg!(w_scale);
            dbg!(h_scale);
            dbg!(min_scale);

            min_scale = min_scale.max(w_scale).max(h_scale);
        }

        let mut target = frame.as_target();

        for object in self.objects.iter() {
            object.sprite.draw(
                graphics::Quad {
                    size: (object.size, object.size),
                    ..Default::default()
                },
                &mut target
                    .transform(graphics::Transformation::translate(center))
                    .transform(graphics::Transformation::translate(
                        object.position.to_homogeneous().xy(),
                    ))
                    .transform(graphics::Transformation::rotate(object.angle))
                    .transform(graphics::Transformation::translate(Vector::from([
                        -object.size / 2.,
                        -object.size / 2.,
                    ]))),
            );
        }
    }
}

#[derive(Debug, Clone)]
struct SpaceObject {
    position: Point,
    velocity: Vector,
    angle: f32,
    mass: f32,
    size: f32,
    sprite: graphics::Image,
}
