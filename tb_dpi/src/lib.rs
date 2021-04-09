extern crate lazy_static;
extern crate mailbox_rs;
use lazy_static::lazy_static;
use mailbox_rs::{
    export_mb_backdoor_dpi,
    mb_channel::*,
    mb_std::{futures::future::join, *},
};
use std::env;
mod rpcs;
use rpcs::*;

type DPIShareMemSpace = MBShareMemSpace<DPIShareMem>;

#[derive(Debug)]
struct DPIShareMem {
    name: String,
    id: u32,
    base: MBPtrT,
    size: MBPtrT,
}

impl MBShareMemBlock for DPIShareMem {
    fn base(&self) -> MBPtrT {
        self.base
    }
    fn size(&self) -> MBPtrT {
        self.size
    }
}

impl MBShareMem for DPIShareMem {
    fn write(&mut self, addr: MBPtrT, data: &[u8]) -> usize {
        extern "C" {
            fn mem_write_bd(id: u32, addr: u64, data: u8);
        }
        unsafe {
            for (i, d) in data.iter().enumerate() {
                mem_write_bd(self.id, addr as u64 + i as u64, *d);
            }
        }
        data.len()
    }
    fn read(&self, addr: MBPtrT, data: &mut [u8]) -> usize {
        extern "C" {
            fn mem_read_bd(id: u32, addr: u64, data: &mut u8);
        }
        unsafe {
            for (i, d) in data.iter_mut().enumerate() {
                mem_read_bd(self.id, addr as u64 + i as u64, d);
            }
        }
        data.len()
    }
}

#[derive(Default)]
struct DPIShareMemParser;
impl MBShareMemParser for DPIShareMemParser {
    type MemType = DPIShareMem;
    fn parse(&self, key: &str, doc: &Yaml) -> Result<Self::MemType, String> {
        Ok(DPIShareMem {
            name: key.to_string(),
            id: doc["id"]
                .as_i64()
                .ok_or("id should be integer!".to_string())? as u32,
            base: doc["base"]
                .as_i64()
                .ok_or("base should be integer!".to_string())? as MBPtrT,
            size: doc["size"]
                .as_i64()
                .ok_or("size should be integer!".to_string())? as MBPtrT,
        })
    }
}

lazy_static! {
    static ref MAILBOX_SYS: MBChannelShareMemSys<DPIShareMemSpace> = {
        let spaces = {
            MBShareMemSpaceBuilder::<DPIShareMem, DPIShareMemParser>::new(
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
        .build()
        .unwrap()
    };
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

fn mb_tick() {
    extern "C" {
        fn mb_step();
    }
    unsafe {
        mb_step();
    }
}

#[no_mangle]
extern "C" fn mb_server_run() {
    async_std::task::block_on(async {
        let w = MAILBOX_SYS.wake(mb_tick);
        let s = MAILBOX_SYS.serve(|server| server.add_cmd(WaitEvent));
        join(w, s).await;
    })
}

#[no_mangle]
extern "C" fn mb_server_run_async() {
    let w = MAILBOX_SYS.wake(mb_tick);
    let s = MAILBOX_SYS.serve(|server| server.add_cmd(WaitEvent));
    async_std::task::spawn(async move {
        join(w, s).await;
    });
}

export_mb_backdoor_dpi!(MAILBOX_SYS);
