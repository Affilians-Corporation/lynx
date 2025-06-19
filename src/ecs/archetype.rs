use lynx_traits::Component;
use crate::data_structures::column::Column;

pub trait Archetype {
    fn new<T: Signature>() -> Self;
    fn map<T: Component>(&self) -> Result<usize, ArchetypeError>;
    fn get<T: Component>(&self, field_position: usize) -> Result<&impl Column, ArchetypeError>;
    fn get_mut<T: Component>(&mut self, field_position: usize) -> Result<&mut impl Column, ArchetypeError>;
    fn get_all<T: Component>(&self) -> Result<&[&impl Column], ArchetypeError>;
    fn get_all_mut<T: Component>(&mut self) -> Result<&[&mut impl Column], ArchetypeError>;
    fn get_entity<T: Signature>(&self, id: usize) -> Result<&T, ArchetypeError>;
    fn get_entity_mut<T: Signature>(&mut self, id: usize) -> Result<&mut T, ArchetypeError>;
    fn query<T: Signature>(&self) -> Result<&[&impl Column], ArchetypeError>;
    fn query_mut<T: Signature>(&mut self) -> Result<&[&mut impl Column], ArchetypeError>;
    fn column_must_resize(&self) -> bool;
    fn insert_component<T: Component>(&mut self, component: &T) -> Result<(), ArchetypeError>;
    fn insert<T: Signature>(&mut self, signature: T);
    fn fill<T: Signature>(&mut self, signature: &T, amount: usize) -> Result<(), ArchetypeError>;
    fn initialize_column<T: Component>(&mut self);
    fn has<T: Component>(&self) -> bool;
    fn get_entity_count(&self) -> usize;
    fn set_entity_count(&mut self, count: usize);
}


/// This represents some combination of components.
///
/// # Purpose
/// It is used for any kind of projection, as well as the archetype insertion parameter type.
/// For API ergonomics, this is implemented for tuples of up to 12 elements, but, for
/// Signatures with more than 12 Components, the trait Signature must be implemented for or derived
/// by a custom struct.
///
/// # Usage
/// Any type that derives this has (or should have) a function for inserting its components into
/// an [`SimpleArchetype`], as well as one function that initializes the Archetype, and then inserts.
/// The trait is necessary as Rust code cannot expand types at runtime, and the Archetype cannot know
/// the types of the [`Component`]s [`Component::DismemberedOutput`] at runtime, so we
/// work in reverse.
///
/// ## Example:
///  ```
///     // Implements Signature automatically
///     use ecs::archetype::Signature;
///     use ecs::archetype::Archetype;
///     use lynx::ecs;
///     use lynx::data_structures::column::Column;
///
///     use lynx_derive::*;
///
///    #[derive(Component)]
///     pub struct A;
///
///     #[derive(Component)]
///     pub struct B;
///
///     #[derive(Component)]
///     pub struct C;
///
///     #[derive(Signature)]
///     #[repr(packed)]
///     pub struct DerivedSignature {
///         a: A, // <-------+
///         b: B, // <-------+---> Must implement Component
///         c: C  // <-------+
///     }
///     // The resulting codegen from the above is the same as the below
///
///     // Implements Signature manually
///     pub struct TestSignature {
///         a: A, // <-------+
///         b: B, // <-------+---> Must implement Component
///         c: C  // <-------+
///     }
///
///     impl Signature for TestSignature {
///         fn insert_components(&self, archetype: &mut impl Archetype) {
///             archetype.insert_component::<A>(&self.a).unwrap();
///             archetype.insert_component::<B>(&self.b).unwrap();
///             archetype.insert_component::<C>(&self.c).unwrap();
///         }
///
///         fn create(archetype: &mut impl Archetype) {
///             if archetype.get_entity_count() == 0 {
///                 archetype.initialize_column::<A>();
///                 archetype.initialize_column::<B>();
///                 archetype.initialize_column::<C>();
///             }
///         }
///
///         fn gen_ids() -> &'static [u32] {
///              &[0, 1, 2]
///         }
///
///          fn bulk(&self, archetype: &mut impl Archetype, times: usize) {
///              for _ in 0..times {
///                  self.insert_components(archetype);
///              }
///          }
///     }
/// ```
///
/// This is the reason we do not rely on Reflection for the most part of Lynx.
/// Queries also receive a [`Signature`] as the query type parameter.
/// ## Example:
/// ```
///     use lynx::ecs::archetype::{Archetype, Signature};
///     use lynx::data_structures::column::Column;
///     use lynx::ecs::simple_archetype::SimpleArchetype;
///
///     use lynx_derive::Signature;
///     use lynx_traits::Component;
///
///     #[derive(Signature)]
///     pub struct TestSignature { a: f32 };
///
///
///     fn some_system() {
///         let mut arch = SimpleArchetype::new::<TestSignature>();
///         // TestSignature must implement Signature, this is a projection
///         // over the Archetype's Table, meaning, query will return only
///         // the components present in TestSignature.
///         // This will error if the Archetype does not have all the
///         // components in TestSignature.
///         let query = arch.query::<TestSignature>();
///     }
/// ```
pub trait Signature {
    fn insert_components(&self, archetype: &mut impl Archetype);
    fn create(archetype: &mut impl Archetype);

    fn gen_ids() -> &'static [u32];

    fn bulk(&self, archetype: &mut impl Archetype, times: usize);
}

#[macro_export]
macro_rules! tuple_impls {
    ( $( $name:ident )+ ) => {
        impl<$($name: Component),+> Signature for ($($name,)+) {
            fn insert_components(&self, archetype: &mut impl Archetype) {
                todo!()
            }
            fn create(archetype: &mut impl Archetype) {
                todo!()
            }
            fn gen_ids() -> &'static [u32] {
                todo!()
            }

            fn bulk(&self, archetype: &mut impl Archetype, times: usize) {
                todo!()
            }
        }
    };
}

//tuple_impls! { A B }
//tuple_impls! { A B C }
//tuple_impls! { A B C D }
//tuple_impls! { A B C D E }
//tuple_impls! { A B C D E F}
//tuple_impls! { A B C D E F G}
//tuple_impls! { A B C D E F G H}
//tuple_impls! { A B C D E F G H I}
//tuple_impls! { A B C D E F G H I J}
//tuple_impls! { A B C D E F G H I J K}
//tuple_impls! { A B C D E F G H I J K L}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchetypeError {
    ComponentNotFound,
    IndexOutOfBounds,
    FieldPositionOverlapsNextComponent,
}