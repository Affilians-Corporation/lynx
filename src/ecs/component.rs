use lynx_derive::*;
use lynx_traits::*;

#[derive(Component, Debug)]
#[repr(packed)]
pub struct Player {
    pub id: u32,
}

#[derive(Component, Debug)]
#[repr(packed)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
#[repr(packed)]
pub struct PhysicsMaterial {
    pub bounciness: f64,
    pub roughness: f64,
}

#[derive(Component)]
#[repr(packed)]
pub struct RigidBody {
    pub material: PhysicsMaterial,
    pub linear_momentum: f64,
    pub angular_momentum: f64,
    pub position: Vector2,
    pub velocity: Vector2
}

#[derive(Component)]
#[repr(packed)]
pub struct Pointers<'a, 'b, 'c> {
    pub one: &'a str,
    pub two: &'a Vec<Vector2>,
    pub three: &'b Box<f32>,
    pub four: &'c String,
}

#[derive(Component)]
#[repr(packed)]
pub struct Enemy;

#[cfg(test)]
pub mod component_test {
    use super::*;

    use std::collections::HashMap;

    #[test]

    fn component_id_test() {
        assert_eq!(f32::id(), 0);
        assert_eq!(Player::id(), 1);
        assert_eq!(Vector2::id(), 2);
        assert_eq!(RigidBody::id(), 4);
        assert_eq!(RigidBody::id(), 4);
    }

    #[test]
    fn component_dismember_test() {
        let player = Player { id: 0 };
        let rigid_body = RigidBody {
            position: Vector2 { x: 0.0, y: 0.0 },
            velocity: Vector2 { x: 0.0, y: 0.0 },
            linear_momentum: 15.0,
            angular_momentum: 7.35,
            material: PhysicsMaterial {
                bounciness: 4.3,
                roughness: 5.5,
            },
        };
        let pointers = Pointers {
            one: "ad",
            two: &vec![Vector2 { x: 0.0, y: 0.0 }],
            three: &Box::new(3.5),
            four: &String::from("abcd"),
        };
        let enemy = Enemy;


        assert_eq!(player.dismember(), 0);
       // assert_eq!(rigid_body.dismember(), ((4.3, 5.5), 15.0, 7.35, (0.0, 0.0), (0.0, 0.0)));
        assert_eq!(Player::dismembered_type_count(), 1);
        assert_eq!(RigidBody::dismembered_type_count(), 8);
        assert_eq!(Pointers::dismembered_type_count(), 4);
        assert_eq!(Enemy::dismembered_type_count(), 0);
        assert_eq!(std::mem::size_of_val(&pointers), 40);
        assert_eq!(std::mem::size_of_val(&enemy), 0);
    }
}
