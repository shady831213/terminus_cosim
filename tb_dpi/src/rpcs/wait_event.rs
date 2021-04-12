use mailbox_rs::{
    mb_channel::*,
    mb_std::{
        async_std::task::{Context, Poll},
        *,
    },
};
pub struct WaitEvent;
impl<RA: MBPtrReader, WA: MBPtrWriter, R: MBPtrResolver<READER = RA, WRITER = WA>>
    MBAsyncRPC<RA, WA, R> for WaitEvent
{
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
            0xffffffff => panic!("{} event num {} not support!", server_name, req.args[1]),
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
impl<RA: MBPtrReader, WA: MBPtrWriter, R: MBPtrResolver<READER = RA, WRITER = WA>>
    CustomAsycRPC<RA, WA, R> for WaitEvent
{
    fn is_me(&self, action: u32) -> bool {
        action == 0x8
    }
}
