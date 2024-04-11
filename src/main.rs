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
    const LIN_ACCELARATION: f32 = 0.001;
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
                    position: Point::from([128.0, 0.0]),
                    velocity: Vector::from([0.0, 2.0]),
                    direction: Vector::from([Self::LIN_ACCELARATION, 0.0]),
                    mass: 1.0,
                    sprite: ship,
                    size: 8.0,
                },
                // Sun
                SpaceObject {
                    position: Point::from([0.0, 0.0]),
                    velocity: Vector::zeros(),
                    direction: Vector::zeros(),
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
                ship.velocity += ship.direction;
            }
            // Turn with A/D
            if input.is_key_pressed(keyboard::KeyCode::A) {
                let rot = nalgebra::Rotation2::new(Self::ROT_ACCELERATION);
                ship.direction = rot.transform_vector(&ship.direction);
            }
            if input.is_key_pressed(keyboard::KeyCode::D) {
                let rot = nalgebra::Rotation2::new(-Self::ROT_ACCELERATION);
                ship.direction = rot.transform_vector(&ship.direction);
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
        let mut pos = Vector::zeros();

        for object in self.objects.iter() {
            pos -= object.mass * object.position.to_homogeneous().xy();
            total_mass += object.mass;
        }

        pos /= total_mass;

        pos += Vector::from([frame.width() / 2., frame.height() / 2.]);

        let mut target = frame.as_target();

        let mut target = target.transform(graphics::Transformation::translate(pos));

        for object in self.objects.iter() {
            object.sprite.draw(
                graphics::Quad {
                    position: object.position - Vector::from([object.size / 2., object.size / 2.]),
                    size: (object.size, object.size),
                    ..Default::default()
                },
                &mut target,
            );
        }
    }
}

#[derive(Debug, Clone)]
struct SpaceObject {
    position: Point,
    velocity: Vector,
    direction: Vector,
    mass: f32,
    size: f32,
    sprite: graphics::Image,
}
