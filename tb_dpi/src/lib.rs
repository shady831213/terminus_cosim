extern crate lazy_static;
extern crate mailbox_rs;
use lazy_static::lazy_static;
use mailbox_rs::{
    export_mb_backdoor_dpi,
    mb_rpcs::*,
    mb_std::{
        futures::future::{join, join_all},
        *,
    },
};
use std::env;
mod rpcs;
use rpcs::*;
mod mem;

type DPIShareMemSpace = MBShareMemSpace<mem::simple::DPIShareMem>;

lazy_static! {
    static ref MAILBOX_SYS: MBChannelShareMemSys<DPIShareMemSpace> = {
        let spaces = {
            MBShareMemSpaceBuilder::<mem::simple::DPIShareMem, mem::simple::DPIShareMemParser>::new(
                &env::var("MEM_CFG_FILE").unwrap(),
            )
            .unwrap()
            .build_shared()
            .unwrap()
            .build_spaces()
            .unwrap()
        };
        MBChannelShareMemBuilder::<DPIShareMemSpace>::new(
            &env::var("MAILBOX_CFG_FILE").unwrap(),
            spaces,
        )
        .unwrap()
        .cfg_channels()
        .unwrap()
        .fs(&env::var("MAILBOX_FS_ROOT").unwrap())
        .unwrap()
        .build()
    };
}

fn mb_tick() -> bool {
    extern "C" {
        fn mb_step();
    }
    unsafe {
        mb_step();
    }
    false
}

extern "C" {
    fn mb_exit(code: u32);
}

#[no_mangle]
extern "C" fn mb_server_run() {
    async_std::task::block_on(async {
        let w = MAILBOX_SYS.wake(mb_tick);
        let s = join_all(
            MAILBOX_SYS
                .serve(|server| server.add_cmd(WaitEvent))
                .into_iter()
                .map(|f| async {
                    let (name, status) = f.await;
                    println!("{} exit!", name);
                    unsafe {
                        mb_exit(status);
                    }
                }),
        );
        join(w, s).await;
    })
}

#[no_mangle]
extern "C" fn mb_server_run_async() {
    let w = MAILBOX_SYS.wake(mb_tick);
    let s = join_all(
        MAILBOX_SYS
            .serve(|server| server.add_cmd(WaitEvent))
            .into_iter()
            .map(|f| async {
                let (name, status) = f.await;
                println!("{} exit!", name);
                unsafe {
                    mb_exit(status);
                }
            }),
    );
    async_std::task::spawn(async move {
        join(w, s).await;
    });
}

export_mb_backdoor_dpi!(MAILBOX_SYS);

#[no_mangle]
pub unsafe extern "C" fn __mb_call(
    ch_name: *const std::os::raw::c_char,
    method: *const std::os::raw::c_char,
    arg_len: u32,
    args: *const MBPtrT,
    status: &mut u32,
) -> MBPtrT {
    extern "C" {
        fn tb_sv_call(
            ch_name: *const std::os::raw::c_char,
            method: *const std::os::raw::c_char,
            arg_len: u32,
            arg0: u32,
            arg1: u32,
            arg2: u32,
            arg3: u32,
            status: &mut u32,
        ) -> MBPtrT;
    }
    tb_sv_call(
        ch_name,
        method,
        arg_len,
        *args as u32,
        *((args as usize + 4) as *const MBPtrT) as u32,
        *((args as usize + 8) as *const MBPtrT) as u32,
        *((args as usize + 0xc) as *const MBPtrT) as u32,
        status,
    )
}
