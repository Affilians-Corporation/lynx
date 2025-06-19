use lynx::ecs::archetype::*;
use lynx::ecs::simple_archetype::*;
use lynx::data_structures::column::*;
use lynx_derive::*;
use lynx_traits::*;

#[derive(Component)]
#[repr(packed)]
pub struct Vector2(f32, f32);

#[derive(Component)]
#[repr(packed)]
pub struct Transform(Vector2, Vector2, Vector2);

#[derive(Component)]
#[repr(packed)]
pub struct Sprite(u32);

#[derive(Component)]
#[repr(packed)]
pub struct Texture(u32);

#[derive(Component)]
#[repr(packed)]
pub struct Audio(u32);

#[derive(Component)]
#[repr(packed)]
pub struct RigidBody {
    pub velocity: Vector2,
    pub acceleration: Vector2,
    pub linear_inertia: f32,
    pub angular_inertia: f32,
    pub mass: f32,
    pub restitution: f32,
    pub friction: f32,
}

#[derive(Component)]
#[repr(packed)]
pub struct RectangularCollider {
    pub width: f32,
    pub height: f32,
}

#[derive(Component)]
#[repr(packed)]
pub struct CircleCollider {
    pub radius: f32,
}

#[derive(Component)]
#[repr(packed)]
pub struct Player;

#[derive(Component)]
#[repr(packed)]
pub struct Enemy;

#[derive(Component)]
#[repr(packed)]
pub struct HP(f32);

#[derive(Component)]
#[repr(packed)]
pub struct Grass;

#[derive(Signature)]
#[repr(packed)]
pub struct PlayerSignature {
    pub marker: Player,
    pub transform: Transform,
    pub sprite: Sprite,
    pub rigid_body: RigidBody,
    pub rectangular_collider: RectangularCollider,
    pub hp: HP,
}

#[derive(Signature)]
#[repr(packed)]
pub struct EnemySignature {
    pub marker: Enemy,
    pub transform: Transform,
    pub sprite: Sprite,
    pub rigid_body: RigidBody,
    pub circle_collider: CircleCollider,
}

#[derive(Signature)]
#[repr(packed)]
pub struct GrassSignature {
    pub marker: Grass,
    pub transform: Transform,
    pub sprite: Sprite,
}

#[derive(Signature)]
#[repr(packed)]
pub struct SimpleSignature {
    pub vector: Vector2,
    pub vector1: Vector2,
    pub vector2: Vector2,
    pub vector3: Vector2,
    pub vector4: Vector2,
    pub vector5: Vector2,
    pub vector6: Vector2
}

const RUNS: u32 = 10_000;

fn main() {
    //bench();
    let mut min = u128::MAX;
    let mut max = 0;
    let mut total = 0;
    for _ in 0..100 {
        let mut vector_archetype = SimpleArchetype::new::<SimpleSignature>();
        let start = std::time::Instant::now();
        for _ in 0..RUNS {
            vector_archetype.insert(SimpleSignature {
                vector: Vector2(0.0, 0.0),
                vector1: Vector2(1.0, 5.5),
                vector2: Vector2(0.155, 111.0),
                vector3: Vector2(10.0, 100.0),
                vector4: Vector2(100.0, 55.5),
                vector5: Vector2(155.2, 122.2),
                vector6: Vector2(102.1, 12202.)
            })
        }
        let end = std::time::Instant::now();
        min = end.duration_since(start).as_nanos().min(min);
        max = end.duration_since(start).as_nanos().max(max);
        total += end.duration_since(start).as_nanos();
    }
    println!(
        "Min: {}\tMax: {}\tAvg: {}",
        min,
        max,
        total as f64 / 100 as f64
    );
}
