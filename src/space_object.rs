use coffee::graphics::{self, Point, Vector};
use coffee::input::keyboard;

/// Describes a physical object in space
#[derive(Debug, Clone)]
pub struct SpaceObject {
    /// 2-D position vector of the object.
    position: Point,
    /// 2-D velocity vector of the object.
    velocity: Vector,
    /// Angle the object is facing, with respect to an (1,0) x-axis vector.
    angle: f32,
    /// The mass of the object, determining its gravitational properties.
    mass: f32,
    /// The size of the object, determining its collision and appearance.
    size: f32,
    /// The image drawn to represent the object.
    sprite: graphics::Image,
    /// If the objects is a controllable space ship, this contains the ships special properties.
    ship: Option<ShipInfo>,
    /// Amount of collisions with other objects this one can survive
    collisions: Option<u8>,
}

/// Describes properties of a space object that is also a ship.
#[derive(Debug, Clone)]
struct ShipInfo {
    /// The cooldown of the ships onboard weapon.
    shot_cd: f32,
    /// The keymap used to control the ship.
    keymap: [keyboard::KeyCode; 4],
}

impl SpaceObject {
    const ROT_ACCELERATION: f32 = 0.05;
    const LIN_ACCELARATION: f32 = 0.001;
    /// Creates a new space objects describing a ship
    pub fn ship(
        position: Point,
        velocity: Vector,
        ship_image: graphics::Image,
        keymap: [keyboard::KeyCode; 4],
    ) -> Self {
        Self {
            position,
            velocity,
            angle: 0.0,
            mass: 1.0,
            size: 16.0,
            sprite: ship_image,
            ship: Some(ShipInfo {
                shot_cd: 0.0,
                keymap,
            }),
            collisions: Some(3),
        }
    }

    /// Creates a new space object describing a celestial body, non-controllable and not a ship.
    pub fn body(
        position: Point,
        velocity: Vector,
        mass: f32,
        size: f32,
        image: graphics::Image,
    ) -> Self {
        Self {
            position,
            velocity,
            angle: 0.0,
            mass,
            size,
            sprite: image,
            ship: None,
            collisions: None,
        }
    }

    /// Returns wether this object is a ship or not.
    pub fn is_ship(&self) -> bool {
        self.ship.is_some()
    }

    /// Reads from the input and controls the ship based on it.
    pub fn interact(
        &mut self,
        input: &keyboard::Keyboard,
        images: &[graphics::Image],
    ) -> Vec<SpaceObject> {
        let mut spawns = Vec::new();

        // If not a ship, nothing to do here.
        if !self.is_ship() {
            return spawns;
        }

        // unwrap info (must be there because of filter)
        let ship_info = self.ship.as_mut().unwrap();
        // Acceleration
        if input.is_key_pressed(ship_info.keymap[0]) {
            self.velocity += nalgebra::Rotation2::new(self.angle).transform_vector(&Vector::x())
                * Self::LIN_ACCELARATION;
            self.sprite = images[1].clone();
        } else {
            self.sprite = images[0].clone();
        }
        // Turning
        if input.is_key_pressed(ship_info.keymap[1]) {
            self.angle += Self::ROT_ACCELERATION;
        }
        if input.is_key_pressed(ship_info.keymap[2]) {
            self.angle -= Self::ROT_ACCELERATION;
        }
        // Weapons
        if input.is_key_pressed(ship_info.keymap[3]) {
            if ship_info.shot_cd <= 0.0 {
                spawns.push(SpaceObject {
                    position: self.position
                        + nalgebra::Rotation2::new(self.angle).transform_vector(&Vector::x())
                            * self.size
                            / 1.5,
                    velocity: self.velocity
                        + nalgebra::Rotation2::new(self.angle).transform_vector(&Vector::x()) * 0.8,
                    angle: self.angle,
                    mass: 0.01,
                    size: 4.0,
                    sprite: images[2].clone(),
                    ship: None,
                    collisions: Some(1),
                });
                ship_info.shot_cd = 1.0;
            }
        }
        // Weapon cooldown
        ship_info.shot_cd = (ship_info.shot_cd - 0.01).max(0.0);
        spawns
    }

    /// Draws the object to its position on the screen.
    pub fn draw(&self, target: &mut graphics::Target) {
        self.sprite.draw(
            graphics::Quad {
                size: (self.size, self.size),
                ..Default::default()
            },
            &mut target
                // Translate by the objects position so the object is focused
                .transform(graphics::Transformation::translate(
                    self.position.to_homogeneous().xy(),
                ))
                // Rotate around center so the object appears rotated
                .transform(graphics::Transformation::rotate(self.angle))
                // Slightly correct by object size so object is centered
                .transform(graphics::Transformation::translate(Vector::from([
                    -self.size / 2.,
                    -self.size / 2.,
                ]))),
        );
    }

    /// Moves the ship by its velocity. If a force is passed, it is first accelerated accordingly.
    pub fn perform_movement(&mut self, force: impl Into<Option<Vector>>) {
        if let Some(f) = force.into() {
            self.velocity += f / self.mass;
        }
        self.position += self.velocity;
    }

    /// The objects position vector as a point.
    pub fn get_position(&self) -> Point {
        self.position
    }

    /// The objects velocity vector.
    #[allow(dead_code)]
    pub fn get_velocity(&self) -> Vector {
        self.velocity
    }

    /// The objects mass.
    pub fn get_mass(&self) -> f32 {
        self.mass
    }

    /// Checks if this object collides with the other object, and if yes, registers a collision on both objects, reducing their allowed collisions by 1 if present.
    pub fn collide(&mut self, other: &mut SpaceObject) {
        if (self.position - other.position).norm() * 2. < self.size + other.size {
            if let Some(c) = &mut self.collisions {
                *c -= 1;
            }
            if let Some(c) = &mut other.collisions {
                *c -= 1;
            }
        }
    }

    /// Returns wether this element can still survive collisions, i.e.
    pub fn collisions_left(&self) -> bool {
        if let Some(c) = self.collisions {
            c > 0
        } else {
            true
        }
    }
}
