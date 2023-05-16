#![no_std]
#![no_main]
extern crate terminus_cosim;
use terminus_cosim::*;
#[export_name = "main"]
fn mbcall_test() -> u32 {
    for _ in 0..5 {
        mbcall!("sv_display2", "method2\0".as_ptr());
    }
    for i in 0..5 {
        mbcall!("sv_display1", "method1\0".as_ptr(), i + 5);
    }
    1
}
