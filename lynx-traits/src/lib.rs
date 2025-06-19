use std::{any::TypeId, collections::HashMap};
use std::sync::OnceLock;
use tabled::Tabled;


/// This is the cornerstone of the engine.
///
/// Every type with Copy implements this, including
///     - u8, u16, u32, u64, u128
///     - i8, i16, i32, i64, i128
///     - f32, f64
///     - bool
///     - Any kind of reference (mutable or not)
///
/// This trait isn't applicable for allocated types, if you really need an allocated type inside a
/// Component, you should store a reference.
///
/// The associated type DismemberedOutput is a tuple representing what types compose the Component,
/// this is the type returned in dismember() and is what gets inserted into a table.
pub trait Component {
    type DismemberedOutput;
    const COUNT: usize;
    fn dismember(self) -> Self::DismemberedOutput;

    fn dismembered_type_count() -> u32;
    fn id() -> u32;

    fn sizes() -> &'static [usize];
}

/// The temporary solution for component ID registration, this will be replaced by some other memory
/// and time-efficient solution.
pub struct ComponentRegistry {
    pub components: HashMap<TypeId, u32>,
    pub next_id: u32,
}

impl ComponentRegistry {
    pub fn id<T: 'static>(&mut self) -> u32 {
        match self.components.get(&TypeId::of::<T>()) {
            Some(id) => *id,
            None => {
                self.components.insert(TypeId::of::<T>(), self.next_id);
                self.next_id += 1;
                *self.components.get(&TypeId::of::<T>()).unwrap()
            }
        }
    }
}

impl<'a, T: Copy + 'a> Component for T {
    type DismemberedOutput = T;
    const COUNT: usize = 1;
    
    fn dismember(self) -> Self::DismemberedOutput {
        self
    }
    fn dismembered_type_count() -> u32 {
        Self::COUNT as u32
    }

    fn id() -> u32 {
        0
    }

    fn sizes() -> &'static [usize] {
        &[size_of::<T>()]
    }
}