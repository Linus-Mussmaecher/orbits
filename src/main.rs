use coffee::graphics::{self, Frame, Point, Vector, Window, WindowSettings};
use coffee::input::keyboard;
use coffee::load::Task;
use coffee::{Game, Result, Timer};

mod space_object;
use space_object::SpaceObject;

fn main() -> Result<()> {
    OrbitsInstance::run(WindowSettings {
        title: String::from("Orbits"),
        size: (1280, 1024),
        resizable: true,
        fullscreen: false,
        maximized: false,
    })
}

/// An instance of the simulation.
struct OrbitsInstance {
    /// All objects being simulated.
    objects: Vec<SpaceObject>,
    /// The current scale of the camera.
    scale: f32,
    /// Selection of cached images.
    image_cache: Vec<graphics::Image>,
}

impl OrbitsInstance {
    const GRAVITY: f32 = 0.1;
}

impl Game for OrbitsInstance {
    type Input = coffee::input::Keyboard;
    type LoadingScreen = coffee::load::loading_screen::ProgressBar;

    fn load(_window: &Window) -> Task<OrbitsInstance> {
        // Load the three images used for entities.
        coffee::load::Join::join((
            graphics::Image::load("./assets/sun.png"),
            graphics::Image::load("./assets/ship.png"),
            graphics::Image::load("./assets/ship_power.png"),
        ))
        .map(|(sun, ship, ship_power)| OrbitsInstance {
            objects: vec![
                // Ships
                SpaceObject::ship(
                    Point::from([256.0, 0.0]),
                    Vector::from([0.0, 0.6]),
                    ship.clone(),
                    [
                        keyboard::KeyCode::W,
                        keyboard::KeyCode::A,
                        keyboard::KeyCode::D,
                        keyboard::KeyCode::S,
                    ],
                ),
                SpaceObject::ship(
                    Point::from([-256.0, 0.0]),
                    Vector::from([0.0, -0.6]),
                    ship.clone(),
                    [
                        keyboard::KeyCode::I,
                        keyboard::KeyCode::J,
                        keyboard::KeyCode::L,
                        keyboard::KeyCode::K,
                    ],
                ),
                // Sun
                SpaceObject::body(
                    Point::from([0.0, 0.0]),
                    Vector::zeros(),
                    1024.,
                    96.,
                    sun.clone(),
                ),
            ],
            scale: 1.0,
            image_cache: vec![ship, ship_power, sun],
        })
    }

    fn interact(&mut self, input: &mut Self::Input, window: &mut Window) {
        // Screen interaction
        if input.was_key_released(keyboard::KeyCode::F11) {
            window.toggle_fullscreen();
        }

        let mut shots = Vec::new();

        // Go over all ships and check for their contollers
        for ship in self
            .objects
            .iter_mut()
            .filter(|possible_ship| possible_ship.is_ship())
        {
            shots.extend(ship.interact(&input, &self.image_cache));
        }

        self.objects.extend(shots);
    }

    fn update(&mut self, _window: &Window) {
        // For every object, calculate the gravitational influence of all other objects on it.
        let forces = self
            .objects
            .iter()
            .map(|object| {
                // For every object...
                let mut f = Vector::zeros();

                // Go over every other object
                for attractor in self.objects.iter() {
                    // Get the distance vector between the two
                    let dist = attractor.get_position() - object.get_position();
                    // If they have are not in the same space, generate a force.
                    // Prevents division by zero and an object attracting itself.
                    if dist.norm() != 0.0 {
                        // The gravitational force between the two is in the direction of the distance vector, proportional to their masses and inversely proportional to the square of the distance vectors length.
                        f += dist.normalize()
                            * Self::GRAVITY
                            * object.get_mass()
                            * attractor.get_mass()
                            / dist.norm_squared();
                    }
                }

                f
            })
            .collect::<Vec<_>>();

        // Then apply accelerations and velocities.
        for (object, &force) in self.objects.iter_mut().zip(forces.iter()) {
            object.perform_movement(Some(force));
        }

        // Delete all objects too far from the origin
        self.objects
            .retain(|object| (object.get_position().to_homogeneous().xy()).norm() <= 1000.)
    }

    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        // Clear the current frame
        frame.clear(graphics::Color::BLACK);

        let mut min_scale: f32 = 1.0;

        let (w, h) = (frame.width(), frame.height());

        for object in self.objects.iter().filter(|obj| obj.is_ship()) {
            let w_scale = object.get_position().x.abs() / w * 2.2;
            let h_scale = object.get_position().y.abs() / h * 2.2;

            min_scale = min_scale.max(w_scale).max(h_scale);
        }

        if min_scale > self.scale || min_scale < self.scale / 2. {
            self.scale = min_scale;
        }

        let mut target = frame.as_target();

        for object in self.objects.iter() {
            object.draw(
                &mut target
                    // Move (0/0) to the center of the camera
                    .transform(graphics::Transformation::translate(Vector::from([
                        w / 2.,
                        h / 2.,
                    ])))
                    // Scale so that all objects are in view
                    .transform(graphics::Transformation::scale(1. / self.scale)),
                // All further transformations are object-specific
            );
        }
    }
}
