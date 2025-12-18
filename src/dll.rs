#![allow(unsupported_calling_conventions)]
use std::ops::Add;
use crate::structs::*;
use crate::console;
use ilhook::x86::{CallbackOption, HookFlags, HookType, Hooker, Registers};
use libmem::{Trampoline, hook_code};
use crate::memory::read;
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
    let mut galaxy = Galaxy::read(galaxy);
    let systems = galaxy.systems.read_val().expect("Failed to get systems");
    let wolf_reiser = systems.as_slice().iter().find(|system| system.read_val().expect("Failed to read system").name.to_string() == "Wolf-Reiser").expect("Failed to find Wolf-Reiser");
    println!("wolf_reiser: {:?}", wolf_reiser.read_val().expect("Failed to read system"));
    let mut wolf_reiser_copy = wolf_reiser.read_val().expect("Failed to read system").deep_copy();
    wolf_reiser_copy.name = AeString::new("Wolf-Reisor");
    wolf_reiser_copy.id = 28;
    wolf_reiser_copy.jumpgate_station_id = wolf_reiser.read_val().expect("Failed to read system").jumpgate_station_id;

    wolf_reiser_copy.pos = wolf_reiser.read_val().expect("Failed to read system").pos + Vector3Int::new(10, 10, 10);

    let mut new_systems = AeArray::new(1);
    new_systems.read_val_mut().unwrap()[0] = wolf_reiser_copy.id;
    wolf_reiser_copy.linked_system_ids = new_systems;
    wolf_reiser_copy.starts_unlocked = true;
    println!("wolf_reiser_copy: {:?}", wolf_reiser_copy);
    println!("wolf_reiser_copy name: {}", wolf_reiser_copy.name.to_string());



    let mut systems_clone = systems.deep_copy();
    systems_clone.push(wolf_reiser_copy.leak_to_heap());
    galaxy.systems = systems_clone.leak_to_heap();
    crate::memory::write(GLOBALS_GALAXY, galaxy.leak_to_heap());
    
    
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
