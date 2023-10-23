#![no_std]
#![no_main]
extern crate terminus_cosim;
use terminus_cosim::*;
#[export_name = "main"]
fn hello_world() -> u32 {
    println!("hello_world sp = {:#x}", get_sp!());
    let hartid = hartid();
    try_fork!(task3, hartid).unwrap();
    let task1_id = try_fork!(task1).unwrap();
    let task2_id = fork_on!(1, task2, hartid, task3, &100usize as *const usize);
    join(task1_id);
    for _ in 0..10 {
        print!("Hello ");
        println!("World!");
    }
    for i in 0..10 {
        cprintln!("Hellow World %d!", i);
    }
    join(task2_id);
    panic!()
}

#[no_mangle]
#[inline(never)]
extern "C" fn task1() {
    println!("This is task1");
}
#[inline(never)]
fn task2(parent: usize, subtask: usize, subargs_ptr: usize) {
    let subargs = unsafe { *(subargs_ptr as *const usize) };
    let task_id = fork!(subtask, subargs);
    println!("This is task2! parent:{}", parent);
    join(task_id);
    println!("join task3 in task2");
}

#[no_mangle]
#[inline(never)]
extern "C" fn task3(parent: usize) {
    for i in 0..10 {
        println!("This is task3! parent:{}, {}", parent, i);
    }
}
