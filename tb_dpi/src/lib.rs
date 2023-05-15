use lazy_static::lazy_static;
use vhost::{
    mailbox_init,
    mailbox_rs::{self, mb_std::*},
    mb_server_run as _mb_server_run, mb_server_run_async as _mb_server_run_async, VHostMb,
};
mod rpcs;
lazy_static! {
    static ref MAILBOX_SYS: VHostMb = mailbox_init(|_| Ok(()), |_| Ok(()), |_| Ok(()));
}

#[no_mangle]
extern "C" fn __mb_exit(code: u32) {
    extern "C" {
        fn mb_exit(code: u32);
    }
    unsafe {
        mb_exit(code);
    }
}

#[no_mangle]
extern "C" fn mb_server_run() {
    _mb_server_run(
        &MAILBOX_SYS,
        || {},
        |server| server.add_cmd(rpcs::WaitEvent),
    )
}

#[no_mangle]
extern "C" fn mb_server_run_async() {
    _mb_server_run_async(
        &MAILBOX_SYS,
        || {},
        |server| server.add_cmd(rpcs::WaitEvent),
    )
}

use mailbox_rs::export_mb_backdoor_dpi;
export_mb_backdoor_dpi!(MAILBOX_SYS);
