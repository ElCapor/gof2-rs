use crate::console;
use ilhook::x86::{CallbackOption, HookFlags, HookType, Hooker, Registers};

/*
   ###    #######  #######  #######  ########  ######   ######  ########  ###### 
  ## ##   ##    ## ##    ## ##    ## ##       ##    ## ##    ## ##       ##    ##
 ##   ##  ##    ## ##    ## ##    ## ##       ##       ##       ##       ##      
##     ## ##    ## ##    ## #######  ######    ######   ######  ######    ###### 
######### ##    ## ##    ## ##  ##   ##             ##       ## ##             ##
##     ## ##    ## ##    ## ##   ##  ##       ##    ## ##    ## ##       ##    ##
##     ## #######  #######  ##    ## ########  ######   ######  ########  ###### 
*/
static GLOBALS_GALAXY_SET_ADDR: usize = 0x0044B312;
static GLOBALS_GALAXY : usize = 0x0060AF3C;





/*
##    ##  ######   ######  ##    ##  ###### 
##    ## ##    ## ##    ## ##   ##  ##    ##
##    ## ##    ## ##    ## ##  ##   ##      
######## ##    ## ##    ## #####     ###### 
##    ## ##    ## ##    ## ##  ##         ##
##    ## ##    ## ##    ## ##   ##  ##    ##
##    ##  ######   ######  ##    ##  ###### 
*/
#[allow(unsupported_calling_conventions)]
unsafe extern "cdecl" fn on_galaxy_set(_: *mut Registers, _: usize) {
    let galaxy = read::<usize>(GLOBALS_GALAXY).expect("Failed to read galaxy ptr");
    println!("Galaxy ptr: {:?}", galaxy);
}


/*
##    ## ######## ######## ##        ###### 
##    ##    ##       ##    ##       ##    ##
##    ##    ##       ##    ##       ##      
##    ##    ##       ##    ##        ###### 
##    ##    ##       ##    ##             ##
##    ##    ##       ##    ##       ##    ##
 ######     ##    ######## ########  ###### 
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



/*
######## ##    ## ######## #######  ##    ##    #######   ######  ######## ##    ## ########
##       ###   ##    ##    ##    ##  ##  ##     ##    ## ##    ##    ##    ###   ##    ##   
##       ####  ##    ##    ##    ##   ####      ##    ## ##    ##    ##    ####  ##    ##   
######   ## ## ##    ##    #######     ##       #######  ##    ##    ##    ## ## ##    ##   
##       ##  ####    ##    ##  ##      ##       ##       ##    ##    ##    ##  ####    ##   
##       ##   ###    ##    ##   ##     ##       ##       ##    ##    ##    ##   ###    ##   
######## ##    ##    ##    ##    ##    ##       ##        ######  ######## ##    ##    ##   
*/
pub fn entry_point() {
    

    let hooker = Hooker::new(
        GLOBALS_GALAXY_SET_ADDR,
        HookType::JmpBack(on_galaxy_set),
        CallbackOption::None,
        0,
        HookFlags::empty(),
    );
    unsafe { hooker.hook().unwrap() };

    console::wait_line_press_to_exit(0);
}
