#![no_std]
use riscv::register::mtvec::TrapMode;
pub use vfw_core::arch::rv::{self, arch::*, clint::*, pmp, riscv, sbi, standard::trap::*, sys::*};
pub use vfw_core::*;
pub use vfw_hal::{embedded_hal, nb};
pub use vfw_mailbox::*;
pub use vfw_primitives::*;
extern crate alloc;
const CLINT_BASE: usize = 0x02000000;

pub static CLINT: Clint = Clint::new(CLINT_BASE, true);

#[export_name = "__wait_ipi"]
pub fn wait_ipi() {
    rv_wait_ipi();
    clint_clear_soft(hartid())
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
pub fn exit(code: u32) -> ! {
    mailbox_exit(code)
}

#[export_name = "__post_init"]
fn post_init() {
    mailbox_init();
}

#[no_mangle]
pub fn __print_args(args: &core::fmt::Arguments) {
    use alloc::string::ToString;
    mailbox_print_str(&args.to_string())
}

#[no_mangle]
pub fn __print_str(s: &str) {
    mailbox_print_str(s)
}
#[no_mangle]
fn __boot_core_init() {
    init_trap(TrapMode::Vectored);
    for i in 1..num_cores() {
        fork_on!(i, init_trap);
    }
}
