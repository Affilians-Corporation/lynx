
pub trait Column {
    fn new<T>() -> Self;
    fn new_with_size<T>(size: usize) -> Self;
    fn new_bytes_with_size(size: usize) -> Self;
    fn resize<T>(&mut self, old_cap: usize, new_cap: usize);
    fn resize_bytes(&mut self, old_cap: usize, new_cap: usize);
    fn insert<T>(&mut self, index: usize, data: T);
    fn get<T>(&self, index: usize) -> T;
    fn fill<T>(&mut self, start: usize, end: usize, data: T);
    fn write_bytes(&mut self, start: usize, data: &[u8]);
}


#[cfg(test)]
pub mod column_test {
    use crate::ecs::component::{PhysicsMaterial, RigidBody, Vector2};
    use super::*;
    use crate::data_structures::simple_column::*;

    #[test]
    pub fn create_column() {
        let mut col = SimpleColumn::new::<u32>();
        col.insert::<u32>(0, u32::MAX);
        col.resize::<u32>(1, 5);
        col.insert::<u32>(1, 152);
        col.insert::<u32>(2, 355);
        col.resize::<u32>(5, 5000);
        col.fill::<u32>(3, 100, 15);
        assert_eq!(col.get::<u32>(0), u32::MAX);
        assert_eq!(col.get::<u32>(1), 152);
        assert_eq!(col.get::<u32>(2), 355);
        assert_eq!(col.get::<u32>(91), 15);

        let mut complex_col = SimpleColumn::new::<RigidBody>();
        complex_col.resize::<RigidBody>(1, 10);
        for i in 0..10 {
            col.insert::<RigidBody>(i, RigidBody { material: PhysicsMaterial { bounciness: 0.0, roughness: 0.0 }, linear_momentum: 16.0, angular_momentum: 15.0, position: Vector2 { x: 5.0, y: 15.23 }, velocity: Vector2 { x: 123.0, y: 98.98 } });
        }

        assert_eq!(size_of_val(&col), 8);
        assert_eq!(size_of_val(&complex_col), 8);
    }
}