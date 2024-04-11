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
    scale: f32,
    ship_image: graphics::Image,
    ship_power_image: graphics::Image,
    sun_image: graphics::Image,
}

impl OrbitsInstance {
    const GRAVITY: f32 = 0.1;
    const ROT_ACCELERATION: f32 = 0.05;
    const LIN_ACCELARATION: f32 = 0.001;
}

impl Game for OrbitsInstance {
    type Input = coffee::input::Keyboard;
    type LoadingScreen = coffee::load::loading_screen::ProgressBar;

    fn load(_window: &Window) -> Task<OrbitsInstance> {
        coffee::load::Join::join((
            graphics::Image::load("./assets/sun.png"),
            graphics::Image::load("./assets/ship.png"),
            graphics::Image::load("./assets/ship_power.png"),
        ))
        .map(|(sun, ship, ship_power)| OrbitsInstance {
            objects: vec![
                // Ship
                SpaceObject {
                    position: Point::from([256.0, 0.0]),
                    velocity: Vector::from([0.0, 0.6]),
                    angle: 0.0,
                    mass: 1.0,
                    sprite: ship.clone(),
                    size: 16.0,
                    ship: Some(ShipInfo {
                        shot_cd: 0.0,
                        fuel: 1.0,
                        keymap: [
                            keyboard::KeyCode::W,
                            keyboard::KeyCode::A,
                            keyboard::KeyCode::D,
                            keyboard::KeyCode::S,
                        ],
                    }),
                },
                // Sun
                SpaceObject {
                    position: Point::from([0.0, 0.0]),
                    velocity: Vector::zeros(),
                    angle: 0.0,
                    mass: 1024.0,
                    sprite: sun.clone(),
                    size: 96.0,
                    ship: None,
                },
            ],
            scale: 1.0,
            ship_image: ship,
            ship_power_image: ship_power,
            sun_image: sun,
        })
    }

    fn interact(&mut self, input: &mut Self::Input, window: &mut Window) {
        // Screen interaction
        if input.was_key_released(keyboard::KeyCode::F11) {
            window.toggle_fullscreen();
        }

        let mut shots = Vec::new();

        for possible_ship in self.objects.iter_mut() {
            if possible_ship.ship.is_some() {
                let ship = possible_ship;
                let ship_info = ship.ship.as_mut().unwrap();
                // Accelerate when W is pressed
                if input.is_key_pressed(ship_info.keymap[0]) {
                    ship.velocity += nalgebra::Rotation2::new(ship.angle)
                        .transform_vector(&Vector::x())
                        * Self::LIN_ACCELARATION;
                    ship.sprite = self.ship_power_image.clone();
                } else {
                    ship.sprite = self.ship_image.clone();
                }
                // Turn with A/D
                if input.is_key_pressed(ship_info.keymap[1]) {
                    ship.angle += Self::ROT_ACCELERATION;
                }
                if input.is_key_pressed(ship_info.keymap[2]) {
                    ship.angle -= Self::ROT_ACCELERATION;
                }
                // Shoot with S
                if ship_info.shot_cd <= 0.0 {
                    if input.is_key_pressed(ship_info.keymap[3]) {
                        shots.push(SpaceObject {
                            position: ship.position + 5.0 * ship.velocity,
                            velocity: ship.velocity
                                + nalgebra::Rotation2::new(ship.angle)
                                    .transform_vector(&Vector::x())
                                    * 0.8,
                            angle: ship.angle,
                            mass: 0.01,
                            size: 4.0,
                            sprite: self.sun_image.clone(),
                            ship: None,
                        });
                        ship_info.shot_cd = 1.0;
                    }
                } else {
                    ship_info.shot_cd = (ship_info.shot_cd - 0.01).max(0.0);
                }
            }
        }

        self.objects.extend(shots);
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

        self.objects
            .retain(|object| (object.position.to_homogeneous().xy()).norm() <= 1000.)
    }

    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        // Clear the current frame
        frame.clear(graphics::Color::BLACK);

        let mut min_scale: f32 = 1.0;

        let (w, h) = (frame.width(), frame.height());

        for object in self.objects.iter().filter(|obj| obj.ship.is_some()) {
            let w_scale = object.position.x.abs() / w * 2.2;
            let h_scale = object.position.y.abs() / h * 2.2;

            min_scale = min_scale.max(w_scale).max(h_scale);
        }

        if min_scale > self.scale || min_scale < self.scale / 2. {
            self.scale = min_scale;
        }

        let mut target = frame.as_target();

        for object in self.objects.iter() {
            object.sprite.draw(
                graphics::Quad {
                    size: (object.size, object.size),
                    ..Default::default()
                },
                &mut target
                    .transform(graphics::Transformation::translate(Vector::from([
                        w / 2.,
                        h / 2.,
                    ])))
                    .transform(graphics::Transformation::scale(1. / self.scale))
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
    ship: Option<ShipInfo>,
}

#[derive(Debug, Clone)]
struct ShipInfo {
    shot_cd: f32,
    fuel: f32,
    keymap: [keyboard::KeyCode; 4],
}
