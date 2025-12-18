use std::ops::{Index, IndexMut};

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
        let mut v: Vec<u16> = s.encode_utf16().collect();
        v.push(0); // Null terminator
        let size = v.len() as u32;
        let text = v.as_mut_ptr();
        std::mem::forget(v); // Leak memory so it persists
        
        Self {
            text,
            size,
        }
    }
    
    pub fn to_string(&self) -> String {
        if self.text.is_null() {
            return String::new();
        }
        unsafe {
            let slice = std::slice::from_raw_parts(self.text, self.size as usize);
            // Find null terminator if it exists within size, or just use size
            let len = slice.iter().position(|&c| c == 0).unwrap_or(self.size as usize);
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
    /// The memory for the array elements and the struct itself is leaked (not managed by Rust anymore).
    pub fn new(count: u32) -> *mut Self where T: Default + Clone {
        let mut vec = vec![T::default(); count as usize];
        let data = vec.as_mut_ptr();
        std::mem::forget(vec); // Leak the data buffer

        let array = Self {
            size: count,
            data,
            size2: count,
        };
        
        // Box the struct and leak it to get a stable pointer
        Box::into_raw(Box::new(array))
    }
    
    pub fn from_vec(vec: Vec<T>) -> *mut Self {
        let mut v = vec;
        let size = v.len() as u32;
        let data = v.as_mut_ptr();
        std::mem::forget(v);
        
        let array = Self {
            size,
            data,
            size2: size,
        };
        
        Box::into_raw(Box::new(array))
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

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct System {
    pub unk0: [u8; 0xC],
    pub name: AeString,
}

#[repr(C)]
#[derive(Debug)]
pub struct Galaxy {
    pub unk0: usize, // uintptr_t
    pub systems: *mut AeArray<*mut System>,
}