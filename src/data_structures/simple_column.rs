use std::alloc::{alloc, realloc, Layout};
use std::ptr::NonNull;
use crate::data_structures::column::Column;

/// Just a different name for a Raw pointer to bytes.
///
/// # Purpose
/// This is the most lightweight vector the market can offer.
/// Instead of having length and capacity tracked here, we
/// offshore this responsibility to the Archetype, reducing the size
/// of the type from 24 bytes to 8 bytes, but, as a trade-off, we must
/// ensure security elsewhere.
///
/// # Usage
/// There are no types here, everything got type-erased into bytes, so, if you
/// want to read types from the Column, you must cast the pointer to that type.
/// ```
///     use lynx::data_structures::column::Column;
///     use lynx::data_structures::simple_column::SimpleColumn;
///
/// fn column_create() {
///         let mut col = SimpleColumn::new::<u32>();
///         col.insert::<u32>(0, 10);
///         println!("{}", col.get::<u32>(0));
///     }
/// ```
///
/// # CAUTION
///
/// This type should NOT be used by end users, but, if you need to, be advised that nothing is
/// checked here, no type is stored, there are absolutely no guard rails (we need this
/// for performance reasons).
#[derive(Debug)]
pub struct SimpleColumn {
    pub data: NonNull<u8>,
}

impl Column for SimpleColumn {
    /// Creates an empty column, with capcity for 1 member (as this is created when
    /// the user calls [`crate::ecs::simple_archetype::SimpleArchetype::insert`]).
    ///
    /// # Usage
    /// ```
    ///     use lynx::data_structures::simple_column::SimpleColumn;
    ///     use lynx::data_structures::column::Column;
    ///
    ///     let mut col = SimpleColumn::new::<u32>();
    /// ```
    /// The layout will be aligned to the type parameter.
    fn new<T>() -> Self {
        Self {
            data: unsafe {NonNull::new(alloc(Layout::array::<T>(4).unwrap())).unwrap()}
        }
    }

    /// Creates an empty [`crate::data_structures::column::Column`] with capacity for storing 'size' elements of type T.
    ///
    /// # Usage
    /// ```
    ///     // Creates a Column that can store 100 u32s.
    ///     use lynx::data_structures::column::Column;
    ///     use lynx::data_structures::simple_column::SimpleColumn;
    ///     let mut col = SimpleColumn::new_with_size::<u32>(100);
    /// ```
    fn new_with_size<T>(size: usize) -> Self {
        Self {
            data: unsafe {NonNull::new(alloc(Layout::array::<T>(size * 4).unwrap())).unwrap()}
        }
    }

    /// Creates an even more Raw vector, a pointer to bytes with 'size' allocated entries.
    ///
    /// # Usage
    /// ```
    ///     use lynx::data_structures::column::Column;
    ///     use lynx::data_structures::simple_column::SimpleColumn;
    ///     let value = 10.0f32 ;
    ///
    ///     // Casts the value into a raw pointer to its bytes.
    ///     let ptr = core::ptr::addr_of!(value) as *const u8;
    ///
    ///     // Turns the raw data read from the pointer into a slice.
    ///     let slice = unsafe{ core::ptr::slice_from_raw_parts(ptr, 4) };
    ///
    ///     // Create a new column that can accommodate 1 f32, which has 4 bytes;
    ///     let mut col = SimpleColumn::new_bytes_with_size(4);
    ///
    ///     // Writes the raw data into the 0th index of the column
    ///     col.write_bytes(0, unsafe {&*slice});
    /// ```
    fn new_bytes_with_size(size: usize) -> Self {
        Self {
            data: unsafe {NonNull::new(alloc(Layout::array::<u8>(size).unwrap())).unwrap()}
        }
    }


    /// Resizes the [`crate::data_structures::column::Column`] from old_cap into new_cap (works even if new_cap < old_cap)
    ///
    /// # Usage
    /// You must provide type information into this.
    /// ```
    ///     use lynx::data_structures::column::Column;
    ///     use lynx::data_structures::simple_column::SimpleColumn;
    ///     let mut col = SimpleColumn::new::<u32>();
    ///     col.resize::<u32>(1, 10);
    /// ```
    fn resize<T>(&mut self, old_cap: usize, new_cap: usize) {
        let new_layout = Layout::array::<T>(new_cap * size_of::<T>()).unwrap();
        let old_layout = Layout::array::<T>(old_cap * size_of::<T>()).unwrap();
        let old_ptr = self.data.as_ptr();

        unsafe {
            let new_ptr = realloc(old_ptr, old_layout, new_layout.size());
            self.data = match NonNull::new(new_ptr) {
                Some(a) => a,
                None  => panic!("Column allocation failed")
            };
        }
    }


    /// Resizes the [`crate::data_structures::column::Column`] from old_cap bytes into new_cap bytes.
    ///
    /// # Usage
    /// No type information is needed, BUT you must account for the size of the concrete
    /// type stored in this column.
    ///
    /// ```
    /// use lynx::data_structures::column::Column;
    /// use lynx::data_structures::simple_column::SimpleColumn;
    /// let value = 10.0f32 ;
    /// let ptr = core::ptr::addr_of!(value) as *const u8;
    /// let slice = unsafe { core::ptr::slice_from_raw_parts(ptr, 4) };
    ///
    /// let mut col = SimpleColumn::new_bytes_with_size(4);
    /// col.write_bytes(0, unsafe {&*slice});
    /// // f32 has 4 bytes, therefore, resizing from 4 to 8 means we
    /// // resize to column to accommodate 2 f32.
    /// col.resize_bytes(4, 8);
    /// col.write_bytes(1, unsafe { &*slice });
    /// ```
    fn resize_bytes(&mut self, old_cap: usize, new_cap: usize) {
        //println!("Old cap: {}\tNext Cap: {}", old_cap, new_cap);
        let new_layout = Layout::array::<u8>(new_cap).unwrap();
        let old_layout = Layout::array::<u8>(old_cap).unwrap();
        let old_ptr = self.data.as_ptr();
        unsafe {
            let new_ptr = realloc(old_ptr, old_layout, new_layout.size());
            self.data = match NonNull::new(new_ptr) {
                Some(a) => a,
                None  => panic!("Column allocation failed")
            };
        }
    }


    /// Inserts a value into the [`crate::data_structures::column::Column`].
    ///
    /// # Usage
    /// Type information must be provided.
    ///
    /// ```
    /// use lynx::data_structures::column::Column;
    /// use lynx::data_structures::simple_column::SimpleColumn;
    /// let mut col = SimpleColumn::new::<u32>();
    /// col.insert::<u32>(0, 10);
    /// ```
    ///
    /// Index is not being checked, the safety of this function is offshored to
    /// [`crate::ecs::simple_archetype::SimpleArchetype`]
    fn insert<T>(&mut self, index: usize, data: T) {
        unsafe {
            core::ptr::write(
                self.data.cast::<T>()
                    .as_ptr()
                    .add(index),
                data)
        };
    }

    /// Retrieves some that stored in 'index'.
    ///
    /// # Usage
    /// This method requires you to specify what type the data will be interpreted as.
    /// Index is not being checked.
    /// ```
    ///     use lynx::data_structures::column::Column;
    /// use lynx::data_structures::simple_column::SimpleColumn;
    ///     let mut col = SimpleColumn::new::<u32>();
    ///     col.insert::<u32>(0, 10);
    ///     assert_eq!(col.get::<u32>(0), 10);
    /// ```
    fn get<T> (&self, index: usize) -> T{
        unsafe {
            core::ptr::read(self.data.as_ptr().add(index * size_of::<T>()) as *const T)
        }
    }

    fn fill<T> (&mut self, start: usize, end: usize, data: T) {
        let extension = end - start;
        unsafe {
            for i in 0..extension {
                core::ptr::copy_nonoverlapping(core::ptr::addr_of!(data), self.data.as_ptr().cast::<T>().add(start + i), i);
            }
        }
    }

    /// Writes 'data' bytes into the [`crate::data_structures::column::Column`] at offset 'start'.
    ///
    /// # Usage
    /// No types are needed here, but you must ensure that this is the right type.
    ///
    /// ```
    ///     use lynx::data_structures::column::Column;
    ///     use lynx::data_structures::simple_column::SimpleColumn;
    ///     let value = 10.0f32 ;
    ///     let ptr = core::ptr::addr_of!(value) as *const u8;
    ///     let slice = unsafe { core::ptr::slice_from_raw_parts(ptr, 4) };
    ///     let mut col = SimpleColumn::new_bytes_with_size(4);
    ///     col.write_bytes(0, unsafe {&*slice});
    /// ```
    fn write_bytes(&mut self, start: usize, data: &[u8]) {
        unsafe {

            core::ptr::copy(data.as_ptr(), self.data.as_ptr().add(start * data.len()), data.len());
        }
    }
}
