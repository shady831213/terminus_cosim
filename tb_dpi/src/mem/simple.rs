use mailbox_rs::{mb_rpcs::*, mb_std::*};
#[derive(Debug)]
pub struct DPIShareMem {
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
pub struct DPIShareMemParser;
impl MBShareMemParser for DPIShareMemParser {
    type MemType = DPIShareMem;
    fn parse(&self, _key: &str, doc: &Yaml) -> Result<Self::MemType, String> {
        Ok(DPIShareMem {
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
