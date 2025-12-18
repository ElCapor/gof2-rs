#![cfg(windows)]
mod console;
mod dll;
pub mod structs;
pub mod memory;

extern crate core;

use std::thread;
use winapi::shared::minwindef;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, UINT, MAX_PATH};
use winapi::shared::d3d9;
use winapi::um::libloaderapi::{LoadLibraryA, GetProcAddress, GetModuleFileNameA};
use winapi::um::consoleapi;
use std::ptr;
use std::ffi::CString;
use std::mem;
use winapi::shared::d3d9::IDirect3D9;

type D3DCreate9 = extern "system" fn(UINT) -> *mut d3d9::IDirect3D9;
type D3DPERFSetOptions = extern "system" fn(DWORD);

static mut H_ORIGINAL: HINSTANCE = ptr::null_mut();
static mut P_DIRECT3D_CREATE9: Option<D3DCreate9> = None;
static mut P_D3D_PERF_SET_OPTIONS: Option<D3DPERFSetOptions> = None;

#[unsafe(no_mangle)]
pub unsafe extern "system" fn D3DPERF_SetOptions(dw_options: DWORD) {
    unsafe {
        match P_D3D_PERF_SET_OPTIONS {
            Some(func) => func(dw_options),
            None => panic!("SetOptions panic")
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Direct3DCreate9(sdk_version: UINT) -> *mut IDirect3D9 {
    unsafe {
        match P_DIRECT3D_CREATE9 {
            Some(func) => func(sdk_version),
            None => panic!("Direct3DCreate9 panic")
        }
    }
}

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: DWORD, reserved: LPVOID) -> BOOL {
    const DLL_PROCESS_ATTACH: DWORD = 1;
    const DLL_PROCESS_DETACH: DWORD = 0;

    match call_reason {
        DLL_PROCESS_ATTACH => initialize(),
        DLL_PROCESS_DETACH => (),
        _ => ()
    }

    minwindef::TRUE
}

fn initialize() {
    unsafe {
        H_ORIGINAL = LoadLibraryA(CString::new("C:\\Windows\\System32\\d3d9.dll").unwrap().as_ptr());
        
        if !H_ORIGINAL.is_null() {
            P_DIRECT3D_CREATE9 = Some(mem::transmute(GetProcAddress(H_ORIGINAL, CString::new("Direct3DCreate9").unwrap().as_ptr())));
            P_D3D_PERF_SET_OPTIONS = Some(mem::transmute(GetProcAddress(H_ORIGINAL, CString::new("D3DPERF_SetOptions").unwrap().as_ptr())));
        } else {
            panic!("Couldn't find d3d9.dll on the system");
        }

        let mut buffer = [0i8; MAX_PATH];
        let len = GetModuleFileNameA(ptr::null_mut(), buffer.as_mut_ptr(), MAX_PATH as u32);
        
        if len > 0 {
            let slice = std::slice::from_raw_parts(buffer.as_ptr() as *const u8, len as usize);
            let path_str = String::from_utf8_lossy(slice).to_lowercase();

            if path_str.contains("gof2launcher.exe") {
                return;
            }
        }

        consoleapi::AllocConsole();
        let _ = thread::spawn(|| {
                dll::entry_point();
            });
        
    }
}
