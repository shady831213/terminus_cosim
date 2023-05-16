use terminus_cosim::*;
struct WaitEvent;
impl MBRpc for WaitEvent {
    type REQ = u32;
    type RESP = u32;
    fn put_req(&self, req: Self::REQ, entry: &mut MBReqEntry) {
        entry.words = 1;
        entry.action = MBAction::OTHER;
        entry.args[0] = 0x8;
        entry.args[1] = req as MBPtrT;
    }
    fn get_resp(&self, resp: &MBRespEntry) -> Self::RESP {
        resp.rets as Self::RESP
    }
}

pub fn mb_wait_event(event: u32) -> u32 {
    mb_sender().send(&WaitEvent, event)
}
