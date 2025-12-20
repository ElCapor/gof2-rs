#![allow(unsupported_calling_conventions, static_mut_refs)]
use std::ops::Add;
use std::thread;
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
static GLOBALS_STATUS: usize = 0x0060AD6C;
static GLOBALS_APP_MANAGER: usize = 0x0060AEFC;


/*
######## ##    ## ##    ##    ######## ########    ###    ######## ##    ## #######  ########  ###### 
##       ##    ## ###   ##    ##       ##         ## ##      ##    ##    ## ##    ## ##       ##    ##
##       ##    ## ####  ##    ##       ##        ##   ##     ##    ##    ## ##    ## ##       ##      
######   ##    ## ## ## ##    ######   ######   ##     ##    ##    ##    ## #######  ######    ###### 
##       ##    ## ##  ####    ##       ##       #########    ##    ##    ## ##  ##   ##             ##
##       ##    ## ##   ###    ##       ##       ##     ##    ##    ##    ## ##   ##  ##       ##    ##
##        ######  ##    ##    ##       ######## ##     ##    ##     ######  ##    ## ########  ###### 
*/


pub fn patch_news()
{
    let app_manager = read::<*mut AppManager>(GLOBALS_APP_MANAGER).expect("Failed to get app manager").read_val_mut().unwrap();
    let mod_station = app_manager.modules.read_val_mut().expect("Failed to get modules").mod_station.read_val_mut().expect("Failed to read station");
    let mod_station_news = mod_station.news.read_val_mut().expect("Failed to read news");
    mod_station_news.news.size = 2;
}


/*
 ######     ###    ##          ###    ##    ## ##    ##
##    ##   ## ##   ##         ## ##    ##  ##   ##  ## 
##        ##   ##  ##        ##   ##    ####     ####  
##  #### ##     ## ##       ##     ##    ##       ##   
##    ## ######### ##       #########   ####      ##   
##    ## ##     ## ##       ##     ##  ##  ##     ##   
 ######  ##     ## ######## ##     ## ##    ##    ##   
*/
pub fn patch_visibilities()
{
    // visibilities = status + 0x24
    let status = read::<usize>(GLOBALS_STATUS).expect("Failed to get status");
    let mut visibilities = read::<*mut AeArray<bool>>(status.add(0x24)).expect("Failed to get visibilities").read_val_mut().unwrap();
    visibilities.push(true);
    for visibility in visibilities.iter_mut() {
        *visibility = true;
    }
    
}

// patch star map to show N systems
pub fn patch_star_map(n: u8)
{
    let push_27 = 0x004CE771;
    // replace push_27 with push_n
    // original = 6A 1B
    // target = 6A n
    // modify memory protection before writing
    let old_protection = crate::memory::set_protection(push_27, 2, windows::Win32::System::Memory::PAGE_EXECUTE_READWRITE.0).expect("Failed to change protection for page");
    crate::memory::write::<u8>(push_27 + 0x1, n).expect("Failed to write memory");
    crate::memory::set_protection(push_27, 2, old_protection).expect("Failed to change protection for page");
}

pub fn patch_load_stations()
{
    let patch1 = 0x004089B9; // 6D TO 6E

    let old_protection = crate::memory::set_protection(patch1, 1, windows::Win32::System::Memory::PAGE_EXECUTE_READWRITE.0).expect("Failed to change protection for page");
    crate::memory::write::<u8>(patch1, 0x6E).expect("Failed to write memory");
    crate::memory::set_protection(patch1, 1, old_protection).expect("Failed to change protection for page");

    let patch2 = 0x408DFE; // 6D TO 6E
    let old_protection = crate::memory::set_protection(patch2, 1, windows::Win32::System::Memory::PAGE_EXECUTE_READWRITE.0).expect("Failed to change protection for page");
    crate::memory::write::<u8>(patch2, 0x6E).expect("Failed to write memory");
    crate::memory::set_protection(patch2, 1, old_protection).expect("Failed to change protection for page");
}

pub fn inject_system()
{
    let galaxy = read::<usize>(GLOBALS_GALAXY).expect("Failed to get galaxy");
    let mut galaxy = Galaxy::read(galaxy);
    let systems = galaxy.systems.read_val().expect("Failed to get systems");
    let wolf_reiser = systems.as_slice().iter().find(|system| system.read_val().expect("Failed to read system").name.to_string() == "Wolf-Reiser").expect("Failed to find Wolf-Reiser");
    println!("wolf_reiser: {:?}", wolf_reiser.read_val().expect("Failed to read system"));
    let mut wolf_reiser_copy = wolf_reiser.read_val().expect("Failed to read system").deep_copy();
    wolf_reiser_copy.name = AeString::new("Custom System");
    wolf_reiser_copy.id = 27;
    wolf_reiser_copy.jumpgate_station_id = 30;//wolf_reiser.read_val().expect("Failed to read system").jumpgate_station_id;

    wolf_reiser_copy.pos = Vector3Int::new(80, 30, 47);

    let mut new_systems = AeArray::new(1);
    new_systems.read_val_mut().unwrap()[0] = wolf_reiser_copy.id;
    wolf_reiser_copy.linked_system_ids = new_systems;
    wolf_reiser_copy.starts_unlocked = true;
    println!("wolf_reiser_copy: {:?}", wolf_reiser_copy);
    println!("wolf_reiser_copy name: {}", wolf_reiser_copy.name.to_string());

    let new_stations = AeArray::new(1);
    new_stations.read_val_mut().unwrap()[0] = 109;
    wolf_reiser_copy.station_ids = new_stations;



    let mut systems_clone = systems.deep_copy();
    systems_clone.push(wolf_reiser_copy.leak_to_heap());
    galaxy.systems = systems_clone.leak_to_heap();
    let idk_stations = vec![0u8; 109].as_mut_ptr();
    std::mem::forget(idk_stations);
    //galaxy.stations = idk_stations;

    crate::memory::write(GLOBALS_GALAXY, galaxy.leak_to_heap());
    patch_star_map(28);

    patch_load_stations();
    
    
    //let systems = read::<usize>(galaxy.add(0x4)).expect("Failed to get systems");
    //let systemsSize = read::<usize>(galaxy.add(0x8)).expect("Failed to get systems size");
    //let systemsArray = read::<usize>(systems.add(0x4)).expect("Failed to get systems array");
    //println!("systemsArray: {:X}", systemsArray);

}


static GLOBAL_LOAD_STATION_FROM_ID: usize = 0x408880;
static GLOBAL_LOAD_STATIONS_FROM_SYSTEM: usize = 0x408C4B;




pub type LoadStationFromIdFn = unsafe extern "stdcall" fn(_: *mut u16) -> usize;
pub type LoadStationsFromSystemFn = unsafe extern "stdcall" fn(_: *mut System) -> usize;

static mut LOAD_STATION_FROM_ID_ORIG: Option<LoadStationFromIdFn> = None;
static mut LOAD_STATIONS_FROM_SYSTEM_ORIG: Option<LoadStationsFromSystemFn> = None;


#[repr(C)]
#[derive(Clone)]
pub struct Station {
    pub name: AeString,
    pub id: usize,
    pub system_id: usize,
    pub unk0: usize,
    pub tex_id: usize,
    pub unk1: usize,
    pub tech_level: usize,
    pub unk2: [u8; 20]
}

impl Default for Station {
    fn default() -> Self {
        Self {
            name: AeString::new(""),
            id: 0,
            system_id: 0,
            unk0: 0,
            tex_id: 0,
            unk1: 0,
            tech_level: 0,
            unk2: [0; 20]
        }
    }
}

static mut STATIONS: Vec<*mut Station> = Vec::new();

pub unsafe extern "stdcall" fn load_station_from_id(id: *mut u16) -> usize {
    unsafe { 
        if STATIONS.iter().any(|station| station.read_val().unwrap().id == id.read_val().unwrap() as usize) {
        let result = AeArray::<Station>::new(1);
        result.read_val_mut().unwrap()[0] = STATIONS.iter().find(|station| station.read_val().unwrap().id == id.read_val().unwrap() as usize).unwrap().read_val().unwrap().clone();
        return result as usize;
    }
    }

    let result = unsafe {
        LOAD_STATION_FROM_ID_ORIG.as_ref().unwrap()(id)
    };
    //println!("load_station_from_id param: {:X}", id);
    println!("load_station_from_id: {:X}", result);
    result
}

pub unsafe extern "stdcall" fn load_stations_from_system(system: *mut System) -> usize {

    let sys: System = system.read_val().expect("Failed to read system");
    unsafe {
        // collect all stations with system id equal to sys.id
        let stations = STATIONS.iter().filter(|station| station.read_val().unwrap().system_id == sys.id as usize).collect::<Vec<_>>();
        if !stations.is_empty() {
            let result = AeArray::<Station>::new(stations.len() as u32);
            for i in 0..stations.len() {
                result.read_val_mut().unwrap()[i] = stations[i].read_val().unwrap().clone();
            }
            println!("Return {:X}", result as usize);
            return result as usize;
        }
    }

    let result = unsafe {
        LOAD_STATIONS_FROM_SYSTEM_ORIG.as_ref().unwrap()(system)
    };


    println!("load_stations_from_system param: {:X}", system as usize);
    println!("load_stations_from_system: {:X}", result);





    result
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
    
    //console::wait_line();
    // We can now inject a system ??
    inject_system();

    // thread::spawn(||{
    //     std::thread::sleep(std::time::Duration::from_secs(10));
    //     patch_visibilities();
    // });

    // thread::spawn(||{
    //     std::thread::sleep(std::time::Duration::from_secs(15));
    //     while (true)
    //     {
    //         patch_news();
    //     }
    // });
    

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
        STATIONS.push(Station {
        name: AeString::new("Frenchie station"),
        id: 109,
        system_id: 27,
        unk0: 0,
        tex_id: 10,
        unk1: 0,
        tech_level: 10,
        unk2: [0; 20],
    }.leak_to_heap());
}
    //crate::memory::suspend_process();
    unsafe { 
        println!("GLOBALS_INIT: {:X}", on_init as usize);
        if let Some(trampoline) = hook_code(GLOBALS_INIT,on_init as usize) {
            GLOBALS_INIT_TRAMPOLINE = Some(trampoline);
        }

        console::wait_line();

        println!("GLOBAL_LOAD_STATION_FROM_ID: {:X}", GLOBAL_LOAD_STATION_FROM_ID);
        println!("GLOBAL_LOAD_STATIONS_FROM_SYSTEM: {:X}", GLOBAL_LOAD_STATIONS_FROM_SYSTEM);

        if let Some(trampoline) = hook_code(GLOBAL_LOAD_STATION_FROM_ID, load_station_from_id as usize) {
            LOAD_STATION_FROM_ID_ORIG = Some(trampoline.callable::<LoadStationFromIdFn>());
        }

        if let Some(trampoline) = hook_code(GLOBAL_LOAD_STATIONS_FROM_SYSTEM, load_stations_from_system as usize) {
            LOAD_STATIONS_FROM_SYSTEM_ORIG = Some(trampoline.callable::<LoadStationsFromSystemFn>());
        }
    }

    //crate::memory::resume_process();
    
    // thread::spawn(||{
    //     std::thread::sleep(std::time::Duration::from_secs(5));
    //     inject_system();
    // });

    //console::wait_line_press_to_exit(0);
}
