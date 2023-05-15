#![no_std]
use riscv::register::mtvec::TrapMode;
pub use vfw_rs::vfw_core::arch::rv::standard::{self, clint::*, pmp, riscv, sbi, sys::*, trap::*};
pub use vfw_rs::vfw_core::*;
pub use vfw_rs::vfw_hal::{embedded_hal, nb};
pub use vfw_rs::vfw_mailbox::*;
pub use vfw_rs::vfw_primitives::*;
const CLINT_BASE: usize = 0x02000000;
#[export_name = "__hart_id"]
fn hart_id() -> usize {
    rv_hart_id()
}

#[export_name = "__save_flag"]
fn save_flag() -> usize {
    rv_save_flag()
}

#[export_name = "__restore_flag"]
fn restore_flag(flag: usize) {
    rv_restore_flag(flag)
}

pub static CLINT: Clint = Clint::new(CLINT_BASE);

#[export_name = "__wait_ipi"]
fn wait_ipi() {
    rv_wait_ipi();
    clint_clear_soft(rv_hart_id())
}

#[export_name = "__send_ipi"]
fn clint_send_soft(hart_id: usize) {
    CLINT.send_soft(hart_id);
}
#[export_name = "__clear_ipi"]
fn clint_clear_soft(hart_id: usize) {
    CLINT.clear_soft(hart_id);
}

#[export_name = "__exit"]
fn exit(code: u32) -> ! {
    mailbox_exit(code)
}

#[export_name = "__pre_init"]
fn pre_init() {
    mailbox_init();
    init_trap(TrapMode::Vectored);
}

#[export_name = "__boot_core_init"]
fn boot_core_init() {
    init_print_str(mailbox_print_str);
    set_arch_task_run(run_task);
}

#[export_name = "__mem_invalid"]
fn mem_invalid(_start: usize, _size: usize) {}

#[export_name = "__mem_flush"]
fn mem_flush(_start: usize, _size: usize) {}

#[export_name = "__mem_wb"]
pub fn mem_wb(_start: usize, _size: usize) {}

#[export_name = "__init_bss"]
fn init_bss(s: *mut u8, n: usize) {
    extern "C" {
        fn mailbox_memset(dest: *mut u8, data: i32, n: usize) -> *mut u8;
    }
    mem_invalid(s as usize, n);
    unsafe {
        mailbox_memset(s, 0, n);
    }
}
