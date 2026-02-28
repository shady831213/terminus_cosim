#![no_std]
#![allow(clippy::missing_safety_doc)]
#![feature(coverage_attribute)]
//workaround for https://github.com/rust-lang/rust/issues/112313
use core::hint::spin_loop;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicU32, Ordering};
//global big lock
static ATOMIC_LOCK_8: AtomicU32 = AtomicU32::new(0);

#[no_mangle]
#[coverage(off)]
pub extern "C" fn __atomic_fetch_add_8(ptr: *mut i64, val: i64) -> i64 {
    // Acquire the spinlock
    while ATOMIC_LOCK_8
        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        spin_loop();
    }

    // Critical section: volatile load/add/store
    let old = unsafe { read_volatile(ptr) };
    let new = old.wrapping_add(val);
    unsafe { write_volatile(ptr, new) };

    // Release the lock
    ATOMIC_LOCK_8.store(0, Ordering::Release);

    old
}

//stubs for gcov

#[no_mangle]
static fopen: usize = 0;

#[no_mangle]
static fread: usize = 0;

#[no_mangle]
static fwrite: usize = 0;

#[no_mangle]
static fseek: usize = 0;

#[no_mangle]
static fclose: usize = 0;

#[no_mangle]
static atoi: usize = 0;

#[no_mangle]
static getenv: usize = 0;

#[no_mangle]
static _impure_ptr: usize = 0;

#[no_mangle]
static vsnprintf: usize = 0;

#[no_mangle]
static sprintf: usize = 0;

#[no_mangle]
static vfprintf: usize = 0;

#[no_mangle]
static fprintf: usize = 0;

#[no_mangle]
static fputs: usize = 0;

#[no_mangle]
static getpid: usize = 0;

#[no_mangle]
static __gcov_merge_add: usize = 0;

#[no_mangle]
#[coverage(off)]
pub unsafe extern "C" fn abort() -> ! {
    extern "C" {
        fn __exit(code: u32) -> !;
    }
    __exit(1)
}
