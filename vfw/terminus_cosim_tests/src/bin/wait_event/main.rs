#![no_std]
#![no_main]
extern crate terminus_cosim;
use terminus_cosim::*;
mod wait_event;
use wait_event::*;
#[export_name = "main"]
fn wait_event_test() -> u32 {
    for i in 0..10 {
        println!("get event {} resp {}!", i, mb_wait_event(i));
    }
    1
}
