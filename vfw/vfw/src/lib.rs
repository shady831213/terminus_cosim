#![no_std]
#![feature(coverage_attribute)]
use riscv::register::mtvec::TrapMode;
pub use vfw_core::arch::rv::{self, arch::*, clint::*, pmp, riscv, sbi, standard::trap::*, sys::*};
pub use vfw_core::*;
pub use vfw_hal::{embedded_hal, nb};
pub use vfw_mailbox::*;
pub use vfw_primitives::*;

extern crate alloc;
#[cfg(feature = "c_cov")]
extern crate rv32cov_stub;

const CLINT_BASE: usize = 0x02000000;
pub static CLINT: Clint = Clint::new(CLINT_BASE, true);

#[export_name = "__wait_ipi"]
#[coverage(off)]
pub fn wait_ipi() {
    rv_wait_ipi();
    clint_clear_soft(hartid())
}

#[export_name = "__send_ipi"]
#[coverage(off)]
fn clint_send_soft(hart_id: usize) {
    CLINT.send_soft(hart_id);
}
#[export_name = "__clear_ipi"]
#[coverage(off)]
fn clint_clear_soft(hart_id: usize) {
    CLINT.clear_soft(hart_id);
}

#[export_name = "__exit"]
#[coverage(off)]
pub fn exit(code: u32) -> ! {
    #[cfg(feature = "c_cov")]
    unsafe {
        extern "C" {
            fn dump_gcov_info();
        }
        if hartid() == 0 || code != 0 {
            // let mut coverage = vec![];
            // Note that this function is not thread-safe! Use a lock if needed.
            // minicov::capture_coverage(&mut coverage).unwrap();
            // let mut f = open("output.profraw", MB_FILE_WRITE);
            // f.write(&coverage);
            dump_gcov_info();
        }
    }
    mailbox_exit(code)
}

#[export_name = "__post_init"]
#[coverage(off)]
fn post_init() {
    mailbox_init();
}

#[no_mangle]
#[coverage(off)]
pub fn __print_args(args: &core::fmt::Arguments) {
    use alloc::string::ToString;
    mailbox_print_str(&args.to_string())
}

#[no_mangle]
#[coverage(off)]
pub fn __print_str(s: &str) {
    mailbox_print_str(s)
}
#[no_mangle]
#[coverage(off)]
fn __boot_core_init() {
    init_trap(TrapMode::Vectored);
    for i in 1..num_cores() {
        fork_on!(i, init_trap);
    }
}

#[cfg(feature = "c_cov")]
#[no_mangle]
#[coverage(off)]
pub extern "C" fn malloc(size: usize) -> *mut u8 {
    use alloc::alloc::Layout;
    unsafe {
        let flag = save_flag();
        // let layout = Layout::from_size_align_unchecked(size.max(1), 8);
        let size_all = size + core::mem::size_of::<usize>();
        let layout = Layout::from_size_align_unchecked(size_all, core::mem::size_of::<usize>());
        let p = alloc::alloc::alloc(layout);
        *(p as *mut usize) = size;
        restore_flag(flag);
        (p as usize + core::mem::size_of::<usize>()) as *mut u8
    }
}

#[cfg(feature = "c_cov")]
#[no_mangle]
#[coverage(off)]
pub extern "C" fn free(ptr: *mut u8) {
    use alloc::alloc::Layout;
    unsafe {
        if ptr.is_null() {
            return;
        }
        let flag = save_flag();
        let ptr_all = (ptr as usize - core::mem::size_of::<usize>()) as *mut u8;
        let size = *(ptr_all as *mut usize);
        let size_all = size + core::mem::size_of::<usize>();
        let layout = Layout::from_size_align_unchecked(size_all, core::mem::size_of::<usize>());
        alloc::alloc::dealloc(ptr_all, layout);
        restore_flag(flag);
    }
}
