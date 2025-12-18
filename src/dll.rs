#![allow(unsupported_calling_conventions)]
use std::ops::Add;
use crate::structs::*;
use crate::console;
use ilhook::x86::{CallbackOption, HookFlags, HookType, Hooker, Registers};
use libmem::{Trampoline, hook_code};
/*
   ###    #######  #######  #######  ########  ######   ######  ########  ###### 
  ## ##   ##    ## ##    ## ##    ## ##       ##    ## ##    ## ##       ##    ##
 ##   ##  ##    ## ##    ## ##    ## ##       ##       ##       ##       ##      
##     ## ##    ## ##    ## #######  ######    ######   ######  ######    ###### 
######### ##    ## ##    ## ##  ##   ##             ##       ## ##             ##
##     ## ##    ## ##    ## ##   ##  ##       ##    ## ##    ## ##       ##    ##
##     ## #######  #######  ##    ## ########  ######   ######  ########  ###### 
*/
static GLOBALS_GALAXY_SET_ADDR: usize = 0x0040AA46;
static GLOBALS_GALAXY : usize = 0x0060AF3C;
static GLOBALS_INIT: usize = 0x0044B20C;


/*
 ######     ###    ##          ###    ##    ## ##    ##
##    ##   ## ##   ##         ## ##    ##  ##   ##  ## 
##        ##   ##  ##        ##   ##    ####     ####  
##  #### ##     ## ##       ##     ##    ##       ##   
##    ## ######### ##       #########   ####      ##   
##    ## ##     ## ##       ##     ##  ##  ##     ##   
 ######  ##     ## ######## ##     ## ##    ##    ##   
*/
pub fn inject_system()
{
    let galaxy = read::<usize>(GLOBALS_GALAXY).expect("Failed to get galaxy");
    let galaxy = read::<Galaxy>(galaxy).expect("Failed to get galaxy");
    println!("galaxy: {:X}", galaxy.systems as usize);

    let systems = read::<AeArray<*mut System>>(galaxy.systems as usize).expect("Failed to get systems");
    println!("systems: {:X}", systems.size);
    for system in systems.as_slice().iter() {
        let system = read::<System>(system.addr() as usize).expect("Failed to get system");
        println!("system: {:?}", system.name.to_string());
    }
    
    //let systems = read::<usize>(galaxy.add(0x4)).expect("Failed to get systems");
    //let systemsSize = read::<usize>(galaxy.add(0x8)).expect("Failed to get systems size");
    //let systemsArray = read::<usize>(systems.add(0x4)).expect("Failed to get systems array");
    //println!("systemsArray: {:X}", systemsArray);

}




/*
##    ##  ######   ######  ##    ##  ###### 
##    ## ##    ## ##    ## ##   ##  ##    ##
##    ## ##    ## ##    ## ##  ##   ##      
######## ##    ## ##    ## #####     ###### 
##    ## ##    ## ##    ## ##  ##         ##
##    ## ##    ## ##    ## ##   ##  ##    ##
##    ##  ######   ######  ##    ##  ###### 
*/
pub type GlobalsInitFn = unsafe extern "stdcall" fn(_:usize, _:usize, _:usize) -> usize;

static mut GLOBALS_INIT_TRAMPOLINE: Option<Trampoline> = None;

unsafe extern "stdcall" fn on_init(arg1:usize, arg2:usize, arg3:usize) -> usize {
    #[allow(static_mut_refs)]
    let result: usize = unsafe {
        GLOBALS_INIT_TRAMPOLINE.as_ref().unwrap().callable::<GlobalsInitFn>()(arg1,arg2,arg3) as usize
    };
    
    console::wait_line();
    // We can now inject a system ??
    inject_system();
    return result;
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
    
    unsafe { 
        println!("GLOBALS_INIT: {:X}", on_init as usize);
        if let Some(trampoline) = hook_code(GLOBALS_INIT,on_init as usize) {
            GLOBALS_INIT_TRAMPOLINE = Some(trampoline);
        }
    }
    

    //console::wait_line_press_to_exit(0);
}
