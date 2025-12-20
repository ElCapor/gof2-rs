#![allow(static_mut_refs)]
/*
##    ## ######## ##    ##  ######  #######  ##    ##
###  ### ##       ###  ### ##    ## ##    ##  ##  ## 
######## ##       ######## ##    ## ##    ##   ####  
## ## ## ######   ## ## ## ##    ## #######     ##   
##    ## ##       ##    ## ##    ## ##  ##      ##   
##    ## ##       ##    ## ##    ## ##   ##     ##   
##    ## ######## ##    ##  ######  ##    ##    ##   
*/

use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_PROTECTION_FLAGS, HeapAlloc, GetProcessHeap, HEAP_ZERO_MEMORY
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows::core::PCSTR;
use std::alloc::{alloc, Layout};

type MallocFn = unsafe extern "C" fn(usize) -> usize;
static mut MALLOC: Option<MallocFn> = None;

/// Allocates memory of size `n` bytes using the game's CRT allocator.
/// This attempts to find `malloc` in common CRT DLLs to ensure compatibility.
pub fn allocate(n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    unsafe {
        // Initialize MALLOC function pointer if not already done
        if MALLOC.is_none() {
            let crt_names = [
                "msvcr100.dll", // Prioritize VS2010 CRT (likely for GoF2)
                "msvcr120.dll",
                "msvcr110.dll",
                "msvcr90.dll",
                "msvcr80.dll",
                "msvcr71.dll",
                "ucrtbase.dll",
                "msvcrt.dll",   // System CRT as last resort
            ];

            let malloc_name = std::ffi::CString::new("malloc").unwrap();
            
            for name in crt_names.iter() {
                let c_name = std::ffi::CString::new(*name).unwrap();
                let handle = GetModuleHandleA(PCSTR(c_name.as_ptr() as *const _)).unwrap_or_default();
                
                if !handle.is_invalid() {
                    if let Some(addr) = GetProcAddress(handle, PCSTR(malloc_name.as_ptr() as *const _)) {
                        println!("Found malloc in {}", name);
                        MALLOC = Some(std::mem::transmute(addr));
                        break;
                    }
                }
            }
            
            if MALLOC.is_none() {
                println!("WARNING: Could not find malloc in any common CRT DLL. Falling back to Process Heap.");
            }
        }

        // Use msvcrt malloc if available
        if let Some(malloc_fn) = MALLOC {
            let ptr = malloc_fn(n);
            if ptr != 0 {
                // Zero initialize the memory
                std::ptr::write_bytes(ptr as *mut u8, 0, n);
            }
            ptr
        } else {
            // Fallback to process heap
            let heap = GetProcessHeap().expect("Failed to get process heap");
            let ptr = HeapAlloc(heap, HEAP_ZERO_MEMORY, n);
            if ptr.is_null() {
                0
            } else {
                ptr as usize
            }
        }
    }
}
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows::Win32::System::Threading::{
    OpenThread, SuspendThread, ResumeThread, GetCurrentProcessId, GetCurrentThreadId, THREAD_SUSPEND_RESUME,
};
use windows::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};

/// Suspends all threads in the current process except the calling thread.
pub fn suspend_process() {
    unsafe {
        let pid = GetCurrentProcessId();
        let tid = GetCurrentThreadId();
        
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0).unwrap_or(INVALID_HANDLE_VALUE);
        if snapshot == INVALID_HANDLE_VALUE {
            return;
        }

        let mut te32 = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };

        if Thread32First(snapshot, &mut te32).is_ok() {
            loop {
                if te32.th32OwnerProcessID == pid && te32.th32ThreadID != tid {
                    if let Ok(h_thread) = OpenThread(THREAD_SUSPEND_RESUME, false, te32.th32ThreadID) {
                        SuspendThread(h_thread);
                        CloseHandle(h_thread);
                    }
                }
                if Thread32Next(snapshot, &mut te32).is_err() {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
    }
}

/// Resumes all threads in the current process except the calling thread.
pub fn resume_process() {
    unsafe {
        let pid = GetCurrentProcessId();
        let tid = GetCurrentThreadId();
        
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0).unwrap_or(INVALID_HANDLE_VALUE);
        if snapshot == INVALID_HANDLE_VALUE {
            return;
        }

        let mut te32 = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };

        if Thread32First(snapshot, &mut te32).is_ok() {
            loop {
                if te32.th32OwnerProcessID == pid && te32.th32ThreadID != tid {
                    if let Ok(h_thread) = OpenThread(THREAD_SUSPEND_RESUME, false, te32.th32ThreadID) {
                        ResumeThread(h_thread);
                        CloseHandle(h_thread);
                    }
                }
                if Thread32Next(snapshot, &mut te32).is_err() {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
    }
}

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

/// Safely write a slice of bytes to the given raw address.
/// Returns `Ok(())` on success, `Err(())` if the address is null or the write fails.
#[inline]
pub fn write_bytes(address: usize, data: &[u8]) -> Result<(), ()> {
    if address == 0 {
        return Err(());
    }
    unsafe {
        let ptr = address as *mut u8;
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
        })) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }
}

/// Sets the memory protection for the specified address and size.
/// Returns the old protection flags on success.
/// 
/// Common flags:
/// - PAGE_EXECUTE (0x10)
/// - PAGE_EXECUTE_READ (0x20)
/// - PAGE_EXECUTE_READWRITE (0x40)
/// - PAGE_READWRITE (0x04)
/// - PAGE_READONLY (0x02)
pub fn set_protection(address: usize, size: usize, new_protect: u32) -> Result<u32, windows::core::Error> {
    unsafe {
        let mut old_protect = PAGE_PROTECTION_FLAGS::default();
        VirtualProtect(
            address as *mut _,
            size,
            PAGE_PROTECTION_FLAGS(new_protect),
            &mut old_protect
        )?;
        Ok(old_protect.0)
    }
}
