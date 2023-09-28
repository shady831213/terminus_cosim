#![no_std]
#![no_main]
extern crate terminus_cosim;
use riscv::register::{mcause, mepc};
use terminus_cosim::*;
#[export_name = "main"]
fn trap_test() -> u32 {
    unsafe { register_exception_handler(my_exp_handler) };
    unsafe {
        core::arch::asm!("ecall");
    }
    0xff
}

#[no_mangle]
extern "C" fn my_exp_handler(
    _ctx: fast_trap::FastContext,
    _a1: usize,
    _a2: usize,
    _a3: usize,
    _a4: usize,
    _a5: usize,
    _a6: usize,
    _a7: usize,
) {
    println!("mepc:{:#x?}", mepc::read());
    println!("exception cuase:{:#x?}", mcause::read().cause());
    mepc::write(mepc::read().wrapping_add(4));
    println!("mepc:{:#x?}", mepc::read());
}
