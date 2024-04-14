use macroquad::prelude::*;

mod space_object;
use space_object::SpaceObject;

#[macroquad::main("Orbits")]
async fn main() {
    let mut instance = OrbitsInstance::new().unwrap();

    loop {
        // Read user input and process it
        instance.interact();
        // Run physics updates
        instance.update();
        // Draw the game to the frame
        instance.draw();

        next_frame().await
    }
}

/// An instance of the simulation.
struct OrbitsInstance {
    /// All objects being simulated.
    objects: Vec<SpaceObject>,
    /// The current camera.
    camera: Camera2D,
    /// Selection of cached images.
    image_cache: Vec<Image>,
}

impl OrbitsInstance {
    /// The gravitic constant governing the attraction of space objects to one another
    const GRAVITY: f32 = 0.1;

    /// Creates a new instance of the simulation
    fn new() -> Result<Self, macroquad::Error> {
        let image_cache = vec![
            Image::from_file_with_format(
                include_bytes!("../assets/ship.png"),
                Some(ImageFormat::Png),
            )?,
            Image::from_file_with_format(
                include_bytes!("../assets/ship_power.png"),
                Some(ImageFormat::Png),
            )?,
            Image::from_file_with_format(
                include_bytes!("../assets/projectile.png"),
                Some(ImageFormat::Png),
            )?,
            Image::from_file_with_format(
                include_bytes!("../assets/sun.png"),
                Some(ImageFormat::Png),
            )?,
            Image::from_file_with_format(
                include_bytes!("../assets/earth.png"),
                Some(ImageFormat::Png),
            )?,
        ];
        Ok(OrbitsInstance {
            objects: vec![
                // Ships
                SpaceObject::ship(
                    Vec2::new(256.0, 0.0),
                    Vec2::new(0.0, 0.6),
                    &image_cache[0],
                    [KeyCode::W, KeyCode::A, KeyCode::D, KeyCode::S],
                ),
                SpaceObject::ship(
                    Vec2::new(-256.0, 0.0),
                    Vec2::new(0.0, -0.6),
                    &image_cache[0],
                    [KeyCode::I, KeyCode::J, KeyCode::L, KeyCode::K],
                ),
                // Sun
                SpaceObject::body(
                    Vec2::new(0.0, 0.0),
                    Vec2::new(0.0, 0.0),
                    1024.,
                    96.,
                    &image_cache[3],
                ),
            ],
            camera: Camera2D::default(),
            image_cache,
        })
    }

    /// Reads user input and lets it act on the simulation.
    fn interact(&mut self) {
        // Screen interaction
        if is_key_released(KeyCode::F11) {
            set_fullscreen(true);
        }
        if is_key_released(KeyCode::Escape) {
            set_fullscreen(false);
        }

        let mut shots = Vec::new();

        // Go over all ships and check for their contollers
        for ship in self
            .objects
            .iter_mut()
            .filter(|possible_ship| possible_ship.is_ship())
        {
            shots.extend(ship.interact(&self.image_cache));
        }

        self.objects.extend(shots);
    }

    /// Performs physics updates such as gravity & collision on the simulation.
    fn update(&mut self) {
        // For every object, calculate the gravitational influence of all other objects on it.
        let forces = self
            .objects
            .iter()
            .map(|object| {
                // For every object...
                let mut f = Vec2::ZERO;

                // Go over every other object
                for attractor in self.objects.iter() {
                    // Get the distance vector between the two
                    let dist = attractor.get_position() - object.get_position();
                    // If they have are not in the same space, generate a force.
                    // Prevents division by zero and an object attracting itself.
                    if dist.length() != 0.0 {
                        // The gravitational force between the two is in the direction of the distance vector, proportional to their masses and inversely proportional to the square of the distance vectors length.
                        f += dist.normalize()
                            * Self::GRAVITY
                            * object.get_mass()
                            * attractor.get_mass()
                            / dist.length_squared();
                    }
                }

                f
            })
            .collect::<Vec<_>>();

        // Then apply accelerations and velocities.
        for (object, &force) in self.objects.iter_mut().zip(forces.iter()) {
            object.perform_movement(Some(force));
        }

        // Now check for collisions
        for i in 0..self.objects.len() {
            for j in (i + 1)..self.objects.len() {
                let (left, right) = self.objects.split_at_mut(j);
                left[i].collide(&mut right[0]);
            }
        }

        // Delete all objects too far from the origin
        self.objects.retain(|object| {
            (object.get_position().length() <= 1000. || object.is_ship())
                && object.collisions_left()
        })
    }

    /// Draws the current state to the screen.
    fn draw(&mut self) {
        // Clear the current frame
        clear_background(BLACK);

        // Draw UI

        set_default_camera();
        draw_text("Ship 1", 0., 20., 12., WHITE);

        // Draw simulation

        let (w, h) = (screen_width(), screen_height());

        let mut scale: f32 = 0.5;

        for object in self.objects.iter().filter(|obj| obj.is_ship()) {
            // 2.2 to leave some padding
            let w_scale = object.get_position().x.abs() / w * 2.2;
            let h_scale = object.get_position().y.abs() / h * 2.2;

            scale = scale.max(w_scale).max(h_scale);
        }

        // Camera is -1 to 1, so width and height 2. Correct by that and the reciprocal of screen width.
        self.camera.zoom = Vec2::new(1. / w, 1. / h) / scale * 2.0;

        set_camera(&self.camera);

        for object in self.objects.iter() {
            object.draw();
        }
    }
}
