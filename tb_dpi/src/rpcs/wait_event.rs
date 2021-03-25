use mailbox_rs::{
    mb_channel::*,
    mb_std::{
        async_std::task::{Context, Poll},
        *,
    },
};
pub struct WaitEvent;
impl<RA: MBPtrReader, R: MBPtrResolver<READER = RA>> MBAsyncRPC<RA, R> for WaitEvent {
    fn poll_cmd(
        &self,
        server_name: &str,
        _r: &R,
        req: &MBReqEntry,
        _cx: &mut Context,
    ) -> Poll<Option<MBRespEntry>> {
        extern "C" {
            fn poll_event(id: u32) -> u32;
        }
        match unsafe { poll_event(req.args[1]) } {
            0 => {
                // println!("{} waiting event num:{}!", server_name, req.args[1]);
                Poll::Pending
            }
            0xffffffff => panic!(format!(
                "{} event num {} not support!",
                server_name, req.args[1]
            )),
            x => {
                let mut resp = MBRespEntry::default();
                resp.words = 1;
                resp.rets = x;
                println!(
                    "{} event num:{} ready, resp {}!",
                    server_name, req.args[1], x
                );
                Poll::Ready(Some(resp))
            }
        }
    }
}
impl<RA: MBPtrReader, R: MBPtrResolver<READER = RA>> CustomAsycRPC<RA, R> for WaitEvent {
    fn is_me(&self, action: u32) -> bool {
        action == 0x8
    }
}
