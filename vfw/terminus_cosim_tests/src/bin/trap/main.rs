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
extern "C" fn my_exp_handler(trap_frame: &mut TrapFrame) {
    println!("mepc:{:#x?}", mepc::read());
    println!("exception cuase:{:#x?}", mcause::read().cause());
    println!("frame:{:#x?}", trap_frame);
    mepc::write(mepc::read().wrapping_add(4));
    println!("mepc:{:#x?}", mepc::read());
}
