use crate::data_structures::column::Column;
use crate::data_structures::simple_column::SimpleColumn;
use crate::ecs::archetype::{Archetype, ArchetypeError, Signature};
use crate::ecs::component::{Player, RigidBody, Vector2};
use lynx_derive::Signature;
use lynx_traits::Component;
use std::alloc::{dealloc, Layout};

/// This represents entities with a specific set of components ([`Signature`])
///
/// # Purpose
/// We could represent all entities in a big ol' Table, but this is inefficient
/// in terms of Cache and Vectorization, therefore, we organize entities in
/// Tables composed of similar entities to achieve faster and more
/// memory-efficient iteration.
///
/// This struct is composed of the entity_count (used for controlling the size of Columns without
/// storing any metadata inside the Column), columns (a column for Component, a Component for field
/// of struct) and a structure that maps Components to columns.
/// This Component mapping takes into consideration the number of fields the Component has.
///
/// ## Example:
/// ```
///     use lynx::ecs::archetype::{Archetype, Signature};
///     use lynx_derive::*;
///     use lynx::ecs::simple_archetype::SimpleArchetype;
///     use lynx::data_structures::column::Column;
///
///     #[derive(Component)]
///     #[repr(packed)]
///     pub struct Vector2(f32, f32);
///
///     #[derive(Component)]
///     #[repr(packed)]
///     pub struct Transform(Vector2, Vector2, Vector2);
///
///     #[derive(Signature)]
///     #[repr(packed)]
///     struct SimpleSignature { vector2: Vector2, transform: Transform }
///
///     fn archetype_mapping() {
///         let mut arch = SimpleArchetype::new::<SimpleSignature>();
///     }
/// ```
pub struct SimpleArchetype {
    // Must become thread-safe
    pub entity_count: u32,
    pub columns: Vec<SimpleColumn>,
    pub type_to_col: &'static [u32],
}

impl Drop for SimpleArchetype {
    fn drop(&mut self) {
        for i in self.columns.iter_mut() {
            unsafe { dealloc(i.data.as_ptr(), Layout::new::<u8>()) };
        }
    }
}

impl Archetype for SimpleArchetype {
    /// Creates an empty [`SimpleArchetype`]
    ///
    /// # CAUTION
    /// This archetype cannot be used immediately after creation, the columns
    /// must be initialized with a [`Signature`].
    /// ## Example
    /// ```
    ///     use lynx::ecs::archetype::{Archetype, Signature};
    ///     use lynx::ecs::simple_archetype::SimpleArchetype;    ///
    ///
    ///     use lynx_derive::Signature;
    ///
    ///
    ///     #[derive(Signature)]
    ///     #[repr(packed)]
    ///     struct TestSignature {}
    ///
    ///     fn archetype_initialization() {
    ///         let mut arch = SimpleArchetype::new::<TestSignature>();
    ///         // arch.columns[0] - Error, no columns are created
    ///         // Initializes the archetype for the Signature
    ///         arch.insert::<TestSignature>(TestSignature {})
    ///     }
    /// ```
    fn new<T: Signature>() -> Self {
        let mut arch = Self {
            entity_count: 0,
            columns: Vec::new(),
            type_to_col: T::gen_ids(),
        };
        T::create(&mut arch);
        arch
    }

    #[inline(always)]
    fn map<T: Component>(&self) -> Result<usize, ArchetypeError> {
        let id = T::id();
        let mut left = 0;
        let mut right = self.type_to_col.len() - 1;
        while left <= right {
            if self.type_to_col[left] == id {
                return Ok(left);
            }
            if self.type_to_col[right] == id {
                return Ok(right);
            }
            left += 1;
            right -= 1;
        }
        Err(ArchetypeError::ComponentNotFound)
    }

    /// Retrieves the column for the Component T as immutable.
    ///
    /// # Usage
    /// The argument 'field_position' determines the property of T, whose Column will be returned
    /// (it works like an offset into the 'columns' property)
    /// ## Example
    /// ```
    ///     use lynx::ecs::archetype::{Archetype, Signature};
    ///     use lynx::ecs::simple_archetype::SimpleArchetype;
    ///     use lynx::data_structures::column::Column;
    ///     use lynx_derive::{Component, Signature};
    ///
    ///     #[derive(Component)]
    ///     #[repr(packed)]
    ///     struct Vector2 { x: f32, y: f32 }
    ///
    ///     #[derive(Component)]
    ///     #[repr(packed)]
    ///     struct Transform { pos: Vector2, rot: Vector2, scale: Vector2 }
    ///
    ///     #[derive(Signature)]
    ///     #[repr(packed)]
    ///     struct SimpleSignature { vector2: Vector2, transform: Transform }
    ///
    ///     fn archetype_get() {
    ///         let mut arch = SimpleArchetype::new::<SimpleSignature>();
    ///         arch.insert_component::<Vector2>(&Vector2 { x: 1.52, y: 1.52 }).unwrap();
    ///         // Returns the field 'x' of the Vector2 inserted into the archetype
    ///         let vector_x_col = arch.get::<Vector2>(0);
    ///     }
    /// ```
    ///
    /// Errors if
    ///  - The Component is not part of the archetype (ComponentNotFound);
    ///  - The sum of the first column for the Component and 'field_position' is greater than
    ///    columns.len() (IndexOutOfBounds) or;
    ///  - This same sum overlaps with the next component stored in the Archetype.
    fn get<T: Component>(
        &self,
        field_position: usize,
    ) -> Result<&impl Column, ArchetypeError> {
        let col = match self.map::<T>() {
            Ok(a) => a,
            Err(_) => return Err(ArchetypeError::ComponentNotFound),
        };
        if col + field_position >= self.columns.len() {
            return Err(ArchetypeError::IndexOutOfBounds);
        }
        if col + field_position >= col + T::dismembered_type_count() as usize {
            return Err(ArchetypeError::FieldPositionOverlapsNextComponent);
        }
        //println!("Column for field at {} component {}: {:?}", field_position, std::any::type_name::<T>(), *col + field_position);
        Ok(&self.columns[col + field_position])
    }

    /// Retrieves the column for the Component T as mutable.
    ///
    /// # Usage
    /// The argument 'field_position' determines the property of T, whose Column will be returned
    /// (it works like an offset into the 'columns' property)
    /// ## Example
    /// ```
    ///     use lynx::ecs::archetype::{Archetype, Signature};
    ///     use lynx::data_structures::column::Column;
    ///     use lynx::ecs::simple_archetype::SimpleArchetype;
    ///     use lynx_derive::{Component, Signature};
    ///
    ///     #[derive(Component)]
    ///     #[repr(packed)]
    ///     struct Vector2 { x: f32, y: f32 }
    ///
    ///     #[derive(Component)]
    ///     #[repr(packed)]
    ///     struct Transform { pos: Vector2, rot: Vector2, scale: Vector2 }
    ///
    ///     #[derive(Signature)]
    ///     #[repr(packed)]
    ///     struct SimpleSignature { vector2: Vector2, transform: Transform }
    ///
    ///     fn archetype_get() {
    ///         let mut arch = SimpleArchetype::new::<SimpleSignature>();
    ///         arch.insert_component::<Vector2>(&Vector2 { x: 1.52, y: 1.52 }).unwrap();
    ///         // Returns the field 'x' of the Vector2 inserted into the archetype
    ///         let vector_x_col = arch.get_mut::<Vector2>(0);
    ///     }
    /// ```
    ///
    /// Errors if
    ///  - The Component is not part of the archetype (ComponentNotFound);
    ///  - The sum of the first column for the Component and 'field_position' is greater than
    ///    columns.len() (IndexOutOfBounds) or;
    ///  - This same sum overlaps with the next component stored in the Archetype.
    fn get_mut<T: Component>(
        &mut self,
        field_position: usize,
    ) -> Result<&mut impl Column, ArchetypeError> {
        let col = match self.map::<T>() {
            Ok(a) => a,
            Err(_) => return Err(ArchetypeError::ComponentNotFound),
        };
        if col + field_position >= self.columns.len() {
            return Err(ArchetypeError::IndexOutOfBounds);
        }
        if col + field_position >= col + T::dismembered_type_count() as usize {
            return Err(ArchetypeError::FieldPositionOverlapsNextComponent);
        }
        Ok(&mut self.columns[col + field_position])
    }

    /// Retrieve all columns for a ['Component'] as immutable.
    ///
    /// # Usage
    /// ```
    ///     use lynx::ecs::archetype::{Archetype, Signature} ;
    ///     use lynx::data_structures::column::Column;
    ///     use lynx::ecs::simple_archetype::SimpleArchetype;
    ///     use lynx_derive::{Component, Signature};
    ///
    ///     #[derive(Component)]
    ///     #[repr(packed)]
    ///     struct Vector2 { x: f32, y: f32 }
    ///
    ///     #[derive(Component)]
    ///     struct Transform { pos: Vector2, rot: Vector2, scale: Vector2 }
    ///
    ///     #[derive(Signature)]
    ///     #[repr(packed)]
    ///     struct SimpleSignature { vector2: Vector2, transform: Transform }
    ///
    ///     fn archetype_get_all() {
    ///         let mut arch = SimpleArchetype::new::<SimpleSignature>();
    ///         arch.insert_component::<Vector2>(&Vector2 { x: 1.52, y: 1.52 }).unwrap();
    ///         arch.insert_component::<Transform>(&Transform {
    ///             pos: Vector2 { x: 1.52, y: 1.52 },
    ///             rot: Vector2 { x: 5.2, y: 15.2 },
    ///             scale: Vector2 { x: 1.52, y: 1.52 }
    ///         }).unwrap();
    ///         // Returns all columns for Transform, in order (as defined in the struct)
    ///         let transform_fields = arch.get_all::<Transform>();
    ///         // Returns all columns for Vector2, in order (as defined in the struct)
    ///         let vector_fields = arch.get_all::<Vector2>();
    ///     }
    /// ```
    ///
    /// Errors if the Component is not part of the Archetype (ComponentNotFound).
    fn get_all<T: Component>(&self) -> Result<&[&impl Column], ArchetypeError> {
        let col = match self.map::<T>() {
            Ok(a) => a,
            Err(_) => return Err(ArchetypeError::ComponentNotFound),
        };
        let slice = &mut [];
        for (index, column) in self.columns.iter().skip(col).enumerate() {
            slice[index] = column;
        }
        Ok(slice)
    }

    /// Retrieve all columns for a ['Component'] as mutable.
    ///
    /// # Usage
    /// ```
    ///     use lynx::ecs::archetype::{Signature, Archetype};
    ///     use lynx::data_structures::column::Column;
    ///     use lynx::ecs::simple_archetype::SimpleArchetype;
    ///     use lynx_derive::{Component, Signature};
    ///
    ///     #[derive(Component)]
    ///     #[repr(packed)]
    ///     struct Vector2 { x: f32, y: f32 }
    ///
    ///     #[derive(Component)]
    ///     #[repr(packed)]
    ///     struct Transform { pos: Vector2, rot: Vector2, scale: Vector2 }
    ///
    ///     #[derive(Signature)]
    ///     #[repr(packed)]
    ///     struct SimpleSignature { vector2: Vector2, transform: Transform }
    ///
    ///     fn archetype_get_all() {
    ///         let mut arch = SimpleArchetype::new::<SimpleSignature>();
    ///         arch.insert_component::<Vector2>(&Vector2 { x: 1.52, y: 1.52 }).unwrap();
    ///         arch.insert_component::<Transform>(&Transform {
    ///             pos: Vector2 { x: 1.52, y: 1.52 },
    ///             rot: Vector2 { x: 5.2, y: 15.2 },
    ///             scale: Vector2 { x: 1.52, y: 1.52 }
    ///         }).unwrap();
    ///         // Returns all columns for Transform, in order (as defined in the struct)
    ///         // as mutable
    ///         let transform_fields = arch.get_all_mut::<Transform>();
    ///         // Returns all columns for Vector2, in order (as defined in the struct)
    ///         // as mutable
    ///         let vector_fields = arch.get_all_mut::<Vector2>();
    ///     }
    /// ```
    ///
    /// Errors if the Component is not part of the Archetype (ComponentNotFound).
    fn get_all_mut<T: Component>(&mut self) -> Result<&[&mut impl Column], ArchetypeError> {
        let col = match self.map::<T>() {
            Ok(a) => a,
            Err(_) => return Err(ArchetypeError::ComponentNotFound),
        };
        let slice = &mut [];
        for (index, column) in self.columns.iter_mut().skip(col).enumerate() {
            slice[index] = column;
        }
        Ok(slice)
    }

    fn get_entity<T: Signature>(&self, id: usize) -> Result<&T, ArchetypeError> {
        todo!()
    }

    fn get_entity_mut<T: Signature>(
        &mut self,
        id: usize
    ) -> Result<&mut T, ArchetypeError> {
        todo!()
    }

    fn query<T: Signature>(&self) -> Result<&[&impl Column], ArchetypeError> {
        Ok(self.get_all::<Vector2>()?)
    }

    fn query_mut<T: Signature>(&mut self) -> Result<&[&mut impl Column], ArchetypeError> {
        Ok(self.get_all_mut::<Vector2>()?)
    }

    /// Binary hack for checking if a number is a power of two.
    ///
    /// # Purpose
    /// This is the way we'll know if we need to resize [`Column`]s, if 'entity_count' is
    /// a power of two, we know that something must be made with the [`Column`]s size.
    /// ## Grow
    /// Imagine entity_count is 15 and columns have 16 spaces, and we insert another entity,
    /// giving us 16 entities, we know then that the Columns must be resized to 32 entities.
    /// ## Shrink
    /// Imagine entity_count is 17 and columns have 64 spaces, if we remove one entity, the columns
    /// should be resized to 32 spaces.
    fn column_must_resize(&self) -> bool {
        if self.entity_count < 4 {
            return false
        }
        self.entity_count != 0 && (self.entity_count & (self.entity_count - 1)) == 0
    }

    /// Inserts one [`Component`] into the archetype.
    ///
    /// # Purpose
    /// This should NEVER be used directly, but as a utility for [`Signature::insert_or_create`],
    /// which will 'iteratively' insert it's components into the archetype.
    ///
    /// # Usage
    /// ```
    ///     use lynx::ecs::archetype::{Archetype, Signature};
    ///     use lynx::data_structures::column::Column;
    ///     use lynx::ecs::simple_archetype::SimpleArchetype;
    ///     use lynx_derive::{Component, Signature};
    ///
    ///     #[derive(Component)]
    ///     #[repr(packed)]
    ///     struct Vector2(f32, f32);
    ///
    ///     #[derive(Signature)]
    ///     struct SimpleSignature{ vector2: Vector2 }
    ///
    ///     fn archetype_insert_component() {
    ///         let mut arch = SimpleArchetype::new::<SimpleSignature>();
    ///         arch.insert_component(&Vector2(9.2, 5.5)).unwrap();
    ///     }
    /// ```
    #[inline(always)]
    fn insert_component<T: Component>(
        &mut self,
        component: &T,
    ) -> Result<(), ArchetypeError> {
        let col = match self.map::<T>() {
            Ok(a) => a,
            Err(_) => return Err(ArchetypeError::ComponentNotFound),
        };
        let sizes = <T as Component>::sizes();

        let mut last_index = 0;
        for (index, value) in sizes.iter().enumerate() {
            let ptr = core::ptr::addr_of!(*component) as *const u8;
            unsafe {
                core::ptr::copy_nonoverlapping(
                    ptr.add(last_index),
                    self.columns[col + index]
                        .data
                        .as_ptr()
                        .add(self.entity_count as usize),
                    *value,
                );
            }
            last_index += *value;
        }
        Ok(())
    }


    /// Inserts an entity ([`Signature`]) into the archetype
    ///
    /// # CAUTION
    /// This should only be called with the [`Signature`] that created the [`SimpleArchetype`]
    ///
    /// # Usage
    /// ```
    ///     use lynx::ecs::archetype::{Archetype, Signature};
    ///     use lynx::data_structures::column::Column;
    ///     use lynx::ecs::simple_archetype::SimpleArchetype;
    ///     use lynx_derive::{Component, Signature};
    ///
    ///     #[derive(Component)]
    ///     #[repr(packed)]
    ///     struct Vector2(f32, f32);
    ///
    ///     #[derive(Signature)]
    ///     pub struct TestSignature {
    ///         a: Vector2,
    ///         b: Vector2,
    ///         c: Vector2
    ///     }
    ///
    ///     fn archetype_insert() {
    ///         // This will automatically create the columns needed for TestSignature
    ///         let mut arch = SimpleArchetype::new::<TestSignature>();
    ///         arch.insert(TestSignature {
    ///                         a: Vector2(1.52, 1.52),
    ///                         b: Vector2(1.52, 1.52),
    ///                         c: Vector2(1.52, 1.52)
    ///                     });
    ///     }
    /// ```
    #[inline(always)]
    fn insert<T: Signature>(&mut self, signature: T) {
        signature.insert_components(self);
    }

    fn fill<T: Signature>(&mut self, signature: &T, amount: usize) -> Result<(), ArchetypeError> {
        todo!()
    }

    /// Creates a [`Column`] and the mapping for the type T.
    ///
    /// # Usage
    /// ```
    ///     use lynx::ecs::archetype::{Archetype, Signature};
    ///     use lynx::ecs::simple_archetype::SimpleArchetype;
    ///     use lynx::data_structures::column::Column;
    ///     use lynx_derive::{Component, Signature};
    ///
    ///     #[derive(Component)]
    ///     #[repr(packed)]
    ///     struct Vector2(f32, f32);
    ///
    ///     #[derive(Signature)]
    ///     #[repr(packed)]
    ///     struct SimpleSignature { vector2: Vector2 };
    ///
    ///     fn archetype_initialize_column() {
    ///         // This will automatically call initialize_column
    ///         let mut arch = SimpleArchetype::new::<SimpleSignature>();
    ///     }
    /// ```
    fn initialize_column<T: Component>(&mut self) {
        //println!("Initializin column for: {:?}", std::any::type_name::<T>());
        let sizes = <T as Component>::sizes();

        for value in sizes.iter() {
            self.columns.push(SimpleColumn::new_bytes_with_size(*value));
        }
    }

    #[inline(always)]
    fn has<T: Component>(&self) -> bool {
        self.map::<T>().is_ok()
    }

    #[inline(always)]
    fn get_entity_count(&self) -> usize {
        self.entity_count as usize
    }

    #[inline(always)]
    fn set_entity_count(&mut self, count: usize) {
        self.entity_count = count as u32;
    }
}

#[derive(Signature)]
struct TestSignature {
    vector2: Vector2,
    player: Player,
    rigid_body: RigidBody,
}

#[cfg(test)]
pub mod archetype_test {
    use super::*;
    use crate::data_structures::column::Column;
    use crate::ecs::component::{PhysicsMaterial, Player, RigidBody, Vector2};
    use crate::ecs::simple_archetype::SimpleArchetype;
    use lynx_derive::*;

    #[derive(Signature)]
    struct TestSignature {
        vector2: Vector2,
        player: Player,
        rigid_body: RigidBody,
    }

    #[test]
    pub fn archetype_insert_signature() {
        let mut arch = SimpleArchetype::new::<TestSignature>();
        println!(
            "Vector: {}\tPlayer {}\t RigidBody: {}",
            Vector2::id(),
            Player::id(),
            RigidBody::id()
        );

        arch.insert(TestSignature {
            vector2: Vector2 { x: 1.52, y: 1.52 },
            player: Player { id: 19 },
            rigid_body: RigidBody {
                position: Vector2 { x: 10.0, y: 15.0 },
                velocity: Vector2 { x: 19.2, y: 0.0 },
                angular_momentum: 10.0,
                linear_momentum: 15.0,
                material: PhysicsMaterial {
                    bounciness: 10.0,
                    roughness: 9.999,
                },
            },
        });

        assert_eq!(arch.get::<Vector2>(0).unwrap().get::<f32>(0), 1.52);
        assert_eq!(arch.get::<Vector2>(1).unwrap().get::<f32>(0), 1.52);
        assert_eq!(arch.get::<Player>(0).unwrap().get::<u32>(0), 19);
        assert_eq!(arch.get::<RigidBody>(0).unwrap().get::<f64>(0), 10.0);
        assert_eq!(arch.get::<RigidBody>(1).unwrap().get::<f64>(0), 9.999);
        assert_eq!(arch.get::<RigidBody>(2).unwrap().get::<f64>(0), 15.0);
        assert_eq!(arch.get::<RigidBody>(3).unwrap().get::<f64>(0), 10.0);
        assert_eq!(arch.get::<RigidBody>(4).unwrap().get::<f32>(0), 10.0);
        assert_eq!(arch.get::<RigidBody>(5).unwrap().get::<f32>(0), 15.0);
        assert_eq!(arch.get::<RigidBody>(6).unwrap().get::<f32>(0), 19.2);
        assert_eq!(arch.get::<RigidBody>(7).unwrap().get::<f32>(0), 0.0);
    }
}
