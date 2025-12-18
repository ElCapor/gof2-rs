/*
##    ## ######## ##    ##  ######  #######  ##    ##
###  ### ##       ###  ### ##    ## ##    ##  ##  ## 
######## ##       ######## ##    ## ##    ##   ####  
## ## ## ######   ## ## ## ##    ## #######     ##   
##    ## ##       ##    ## ##    ## ##  ##      ##   
##    ## ##       ##    ## ##    ## ##   ##     ##   
##    ## ######## ##    ##  ######  ##    ##    ##   
*/
/// Safely read a value of type `T` from the given raw address.
/// Returns `None` if the address is null or the read fails.
#[inline]
pub fn read<T>(address: usize) -> Option<T> {
    if address == 0 {
        return None;
    }
    unsafe {
        let ptr = address as *const T;
        // Basic alignment check: size must be a power of two and address aligned to it.
        let align = core::mem::align_of::<T>();
        if address & (align - 1) != 0 {
            return None;
        }
        // Copy the value to avoid moving potentially non-UnwindSafe data across unwind boundary
        let val = ptr.read();
        // Attempt the copy inside catch_unwind to avoid panics propagating.
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| val)) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}

/// Safely write a value of type `T` to the given raw address.
/// Returns `Ok(())` on success, `Err(())` if the address is null or the write fails.
#[inline]
pub fn write<T>(address: usize, data: T) -> Result<(), ()> {
    if address == 0 {
        return Err(());
    }
    unsafe {
        let ptr = address as *mut T;
        // Basic alignment check: size must be a power of two and address aligned to it.
        let align = core::mem::align_of::<T>();
        if address & (align - 1) != 0 {
            return Err(());
        }
        // Attempt the write inside catch_unwind to avoid panics propagating.
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ptr.write(data);
        })) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }
}