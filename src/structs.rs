use std::{io::Read, ops::{Index, IndexMut, Add}};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AeString {
    pub text: *mut u16,
    pub size: u32,
}

impl Default for AeString {
    fn default() -> Self {
        Self {
            text: std::ptr::null_mut(),
            size: 0,
        }
    }

}

impl AeString {
    pub fn new(s: &str) -> Self {
        let v: Vec<u16> = s.encode_utf16().collect();
        let len = v.len();
        let size_bytes = (len + 1) * 2; // +1 for null terminator, u16 is 2 bytes
        
        let text_addr = crate::memory::allocate(size_bytes);
        let text = text_addr as *mut u16;
        
        if !text.is_null() {
            unsafe {
                let slice = std::slice::from_raw_parts_mut(text, len + 1);
                for (i, c) in v.iter().enumerate() {
                    slice[i] = *c;
                }
                slice[len] = 0; // Null terminator
            }
        }

        Self { text, size: len as u32 } // Size usually excludes null terminator in some counts, but let's check original. Original: v.len().
    }

    pub fn to_string(&self) -> String {
        if self.text.is_null() {
            return String::new();
        }
        unsafe {
            let slice = std::slice::from_raw_parts(self.text, self.size as usize);
            // Find null terminator if it exists within size, or just use size
            let len = slice
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(self.size as usize);
            String::from_utf16_lossy(&slice[..len])
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct AeArray<T> {
    pub size: u32,
    pub data: *mut T,
    pub size2: u32,
}

impl<T> AeArray<T> {
    /// Creates a new AeArray on the heap and returns a raw pointer to it.
    /// The memory is allocated using the process heap (via crate::memory::allocate) to be compatible with the game's allocator.
    pub fn new(count: u32) -> *mut Self
    where
        T: Default + Clone,
    {
        // Allocate buffer for elements
        let size_bytes = (count as usize) * std::mem::size_of::<T>();
        let data_addr = crate::memory::allocate(size_bytes);
        let data = data_addr as *mut T;

        // Initialize elements
        if !data.is_null() && count > 0 {
            unsafe {
                let slice = std::slice::from_raw_parts_mut(data, count as usize);
                for item in slice {
                    std::ptr::write(item, T::default());
                }
            }
        }

        let array = Self {
            size: count,
            data,
            size2: count,
        };

        // Allocate the struct itself on the same heap
        let struct_addr = crate::memory::allocate(std::mem::size_of::<Self>());
        let struct_ptr = struct_addr as *mut Self;
        unsafe {
            std::ptr::write(struct_ptr, array);
        }

        struct_ptr
    }

    pub fn from_vec(vec: Vec<T>) -> *mut Self {
        let count = vec.len() as u32;
        let array_ptr = Self::new(count);
        
        unsafe {
            let array = &mut *array_ptr;
            let slice = array.as_mut_slice();
            for (i, item) in vec.into_iter().enumerate() {
                slice[i] = item;
            }
        }
        
        array_ptr
    }

    /// Returns a slice of the array elements.
    /// Safety: Assumes `data` is valid and `size` is correct.
    pub fn as_slice(&self) -> &[T] {
        if self.data.is_null() {
            return &[];
        }
        unsafe { std::slice::from_raw_parts(self.data, self.size as usize) }
    }

    /// Returns a mutable slice of the array elements.
    /// Safety: Assumes `data` is valid and `size` is correct.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if self.data.is_null() {
            return &mut [];
        }
        unsafe { std::slice::from_raw_parts_mut(self.data, self.size as usize) }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }

    /// Pushes a new element into the array by allocating a new buffer,
    /// deep copying all existing elements, adding the new one, and updating the pointer.
    /// The old buffer is leaked (not freed) to avoid potential crashes if the game still holds references.
    pub fn push(&mut self, element: T) 
    where T: DeepCopy
    {
        unsafe {
            // 1. Calculate new size
            let old_size = self.size as usize;
            let new_size = old_size + 1;
            let new_size_bytes = new_size * std::mem::size_of::<T>();

            // 2. Allocate new buffer compatible with game allocator
            let new_data_addr = crate::memory::allocate(new_size_bytes);
            let new_data = new_data_addr as *mut T;

            if !new_data.is_null() {
                // 3. Deep copy existing elements
                if !self.data.is_null() && old_size > 0 {
                    let old_slice = std::slice::from_raw_parts(self.data, old_size);
                    let new_slice = std::slice::from_raw_parts_mut(new_data, new_size);
                    for i in 0..old_size {
                        new_slice[i] = old_slice[i].deep_copy();
                    }
                }

                // 4. Deep copy and add the new element
                let new_slice = std::slice::from_raw_parts_mut(new_data, new_size);
                new_slice[old_size] = element.deep_copy();

                // 5. Update the struct fields
                self.size = new_size as u32;
                self.size2 = new_size as u32;
                self.data = new_data;
                
                // Old buffer is leaked as requested
            }
        }
    }
}

impl<T> Index<usize> for AeArray<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<T> IndexMut<usize> for AeArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Vector3Int {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Default for Vector3Int {
    fn default() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    
}

impl Vector3Int {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl Add for Vector3Int {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct System {
    pub unk0: [u8; 0xC],
    pub name: AeString,
    pub id: u32,
    pub risk: u32,
    pub faction: u32,
    pub pos: Vector3Int,
    pub jumpgate_station_id: u32,
    pub texture_id: u32,
    pub station_ids: *mut AeArray<u32>,
    pub unk1: [u8; 0x4],
    pub linked_system_ids: *mut AeArray<u32>,
    pub starts_unlocked: bool,
}

#[repr(C)]
#[derive(Debug)]
pub struct Station {

}

#[repr(C)]
#[derive(Debug)]
pub struct Galaxy {
    pub stations: *mut u8,
    pub systems: *mut AeArray<*mut System>,
}

/*
#######  ########    ###    #######  ######## #######   ######  ##    ## #######  ######## #######
##    ## ##         ## ##   ##    ## ##       ##    ## ##    ## ###  ### ##    ##    ##    ##    ##
##    ## ##        ##   ##  ##    ## ##       ##    ## ##    ## ######## ##    ##    ##    ##    ##
#######  ######   ##     ## ##    ## ######   #######  ##    ## ## ## ## #######     ##    #######
##  ##   ##       ######### ##    ## ##       ##  ##   ##    ## ##    ## ##          ##    ##  ##
##   ##  ##       ##     ## ##    ## ##       ##   ##  ##    ## ##    ## ##          ##    ##   ##
##    ## ######## ##     ## #######  ##       ##    ##  ######  ##    ## ##          ##    ##    ##
*/
pub trait RWObject {
    fn read(ptr: usize) -> Self;
    fn write(ptr: usize, obj: Self);
    fn size() -> usize;
}

use crate::memory::{read, write};

// Blanket implementation for ALL types
impl<T: Sized> RWObject for T {
    fn read(ptr: usize) -> Self {
        // Use the type name in the error message for better debugging
        read::<Self>(ptr).expect(&format!("Failed to read type: {}", std::any::type_name::<Self>()))
    }

    fn write(ptr: usize, obj: Self) {
        write(ptr, obj).expect(&format!("Failed to write type: {}", std::any::type_name::<Self>()));
    }

    fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

pub trait PtrRW<T> {
    fn read_val(self) -> Option<T>;
    fn write_val(self, val: T) -> Result<(), ()>;
    /// Returns a mutable reference to the value.
    /// Safety: The pointer must be valid and aligned. The lifetime is unbound ('static) so use with caution.
    fn read_val_mut(self) -> Option<&'static mut T>;
}

impl<T> PtrRW<T> for *mut T {
    fn read_val(self) -> Option<T> {
        crate::memory::read(self as usize)
    }

    fn write_val(self, val: T) -> Result<(), ()> {
        crate::memory::write(self as usize, val)
    }

    fn read_val_mut(self) -> Option<&'static mut T> {
        if self.is_null() {
            None
        } else {
            unsafe { Some(&mut *self) }
        }
    }
}

pub trait DeepCopy {
    fn deep_copy(&self) -> Self;
}

// Primitives: Deep copy is just a copy
impl DeepCopy for u8 { fn deep_copy(&self) -> Self { *self } }
impl DeepCopy for u16 { fn deep_copy(&self) -> Self { *self } }
impl DeepCopy for u32 { fn deep_copy(&self) -> Self { *self } }
impl DeepCopy for u64 { fn deep_copy(&self) -> Self { *self } }
impl DeepCopy for i32 { fn deep_copy(&self) -> Self { *self } }
impl DeepCopy for usize { fn deep_copy(&self) -> Self { *self } }
impl DeepCopy for bool { fn deep_copy(&self) -> Self { *self } }
impl<T: DeepCopy + Copy, const N: usize> DeepCopy for [T; N] {
    fn deep_copy(&self) -> Self {
        *self // Arrays of primitives are Copy
    }
}

impl DeepCopy for Vector3Int {
    fn deep_copy(&self) -> Self {
        self.clone()
    }
}

impl DeepCopy for AeString {
    fn deep_copy(&self) -> Self {
        if self.text.is_null() {
            return Self::default();
        }
        
        unsafe {
            // Determine size including null terminator if possible, or trust .size
            // Here we trust .size and assume it includes everything needed or is just capacity
            // Usually for AeString, size is length.
            
            let len = self.size as usize;
            let size_bytes = (len + 1) * 2;
            
            let new_text_addr = crate::memory::allocate(size_bytes);
            let new_text = new_text_addr as *mut u16;
            
            if !new_text.is_null() {
                let src_slice = std::slice::from_raw_parts(self.text, len);
                let dst_slice = std::slice::from_raw_parts_mut(new_text, len + 1);
                
                for i in 0..len {
                    dst_slice[i] = src_slice[i];
                }
                // Ensure null termination
                dst_slice[len] = 0;
            }
            
            Self {
                text: new_text,
                size: self.size,
            }
        }
    }
}

impl<T> DeepCopy for AeArray<T> where T: DeepCopy + Default + Clone {
    fn deep_copy(&self) -> Self {
        if self.data.is_null() || self.size == 0 {
             return Self { size: 0, data: std::ptr::null_mut(), size2: 0 };
        }
        
        unsafe {
            let slice = self.as_slice();
            let mut new_vec = Vec::with_capacity(self.size as usize);
            
            for item in slice {
                new_vec.push(item.deep_copy());
            }
            
            let new_data = new_vec.as_mut_ptr();
            let new_size = new_vec.len() as u32;
            std::mem::forget(new_vec); // Leak
            
            Self {
                size: new_size,
                data: new_data,
                size2: new_size,
            }
        }
    }
}

// For pointer types: *mut T
// If we have a pointer, deep copy usually means "allocate new T, copy content, return pointer to new T"
// BUT, implementing for generic *mut T is risky because we don't know if it's an array or single item.
// We will implement specific handling in structs that have pointers.

impl DeepCopy for *mut System {
    fn deep_copy(&self) -> Self {
        if self.is_null() {
            return std::ptr::null_mut();
        }
        unsafe {
            let original = &**self;
            let copy = original.deep_copy();
            Box::into_raw(Box::new(copy))
        }
    }
}

pub trait HeapAlloc: Sized {
    fn leak_to_heap(self) -> *mut Self;
}

impl<T> HeapAlloc for T {
    fn leak_to_heap(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }
}

impl DeepCopy for System {
    fn deep_copy(&self) -> Self {
        let mut new_sys = self.clone(); // Shallow copy first
        
        new_sys.name = self.name.deep_copy();
        
        // Handle station_ids (pointer to AeArray)
        if !self.station_ids.is_null() {
             unsafe {
                 let old_array = &*self.station_ids;
                 let new_array_struct = old_array.deep_copy();
                 new_sys.station_ids = Box::into_raw(Box::new(new_array_struct));
             }
        }
        
        // Handle linked_system_ids
        if !self.linked_system_ids.is_null() {
             unsafe {
                 let old_array = &*self.linked_system_ids;
                 let new_array_struct = old_array.deep_copy();
                 new_sys.linked_system_ids = Box::into_raw(Box::new(new_array_struct));
             }
        }
        
        new_sys
    }
}

impl DeepCopy for Galaxy {
    fn deep_copy(&self) -> Self {
        let mut new_galaxy = Self { stations: self.stations, systems: std::ptr::null_mut() };
        
        if !self.systems.is_null() {
            unsafe {
                 let old_array = &*self.systems;
                 let new_array_struct = old_array.deep_copy();
                 new_galaxy.systems = Box::into_raw(Box::new(new_array_struct));
            }
        }
        
        new_galaxy
    }
}




pub struct ModStationNewsStruct {
    pub unk0: [u8; 0x14],
    pub news: AeString,
}
pub struct ModStation {
    pub unk0: [u8; 0x18],
    pub news: *mut ModStationNewsStruct,
}

pub struct MGame {

}

pub struct ModMainMenu {

}

pub struct ModTitle {
    
}

pub struct ModulesList {
    pub m_game: *mut MGame,
    pub mod_main_menu: *mut ModMainMenu,
    pub mod_station: *mut ModStation,
    pub mod_title: *mut ModTitle,
}


pub struct AppManager
{
    pub unk0: [u8; 0x44],
    // TODO: Set this as an aearray way later
    pub modules: *mut ModulesList,
}