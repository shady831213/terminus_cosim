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
    println!("ecall exit!");
    0xff
}

#[no_mangle]
extern "C" fn my_exp_handler(ctx: &mut FlowContext) {
    println!("mepc:{:#x?}, pc:{:#x}", mepc::read(), ctx.pc);
    println!("exception cuase:{:#x?}", mcause::read().cause());
    ctx.pc += 4;
}
