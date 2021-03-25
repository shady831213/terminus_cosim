use paste::paste;
use std::rc::Rc;
use terminus::devices::bus::{Bus, TerminusBus};
use terminus::memory::{prelude::*, region::*, MemInfo};
use terminus::space::Space;

pub struct CoreBus {
    local_space: Space,
    sys_bus: Rc<TerminusBus>,
}

impl CoreBus {
    pub fn new(sys_bus: &Rc<TerminusBus>, name:String, hartid:u32, ilm_info: MemInfo, dlm_info: MemInfo) -> CoreBus {
        let mut space = Space::new();
        let ilm = Box::new(ExtBus {
            name: format!("{}.ilm",name),
            id: hartid << 4,
            base: ilm_info.base,
            size: ilm_info.size,
        });
        let dlm = Box::new(ExtBus {
            name: format!("{}.dlm",name),
            id: (hartid << 4) + 1,
            base: dlm_info.base,
            size: dlm_info.size,
        });
        space
            .add_region("ilm", &Region::remap(ilm_info.base, &Region::io(0, ilm.size, ilm)))
            .unwrap();
        space
            .add_region("dlm", &Region::remap(dlm_info.base, &Region::io(0, dlm.size, dlm)))
            .unwrap();
        CoreBus {
            local_space: space,
            sys_bus: sys_bus.clone(),
        }
    }

    fn try_read_local(&self, addr: &u64, data: *mut u8, len: usize) -> Result<(), u64> {
        self.local_space
            .read_bytes(addr, unsafe { std::slice::from_raw_parts_mut(data, len) })?;
        Ok(())
    }

    fn try_write_local(&self, addr: &u64, data: *const u8, len: usize) -> Result<(), u64> {
        self.local_space
            .write_bytes(addr, unsafe { std::slice::from_raw_parts(data, len) })?;
        Ok(())
    }

    fn is_local(&self, addr: &u64) -> bool {
        self.local_space.get_region_by_addr(addr).is_ok()
    }
}

macro_rules! corebus_add_access {
    ($($t:ty),+ ) => {
        $(
            corebus_add_access!(@write, $t);
            corebus_add_access!(@read, $t);
        )+
    };
    (@write, $t:ty) => {
        paste! {
            fn [<write_ $t>](&self, addr: &u64, data: &$t) -> Result<(), u64> {
                if self.try_write_local(addr, data as *const $t as *const u8, std::mem::size_of::<$t>()).is_err() {
                    self.sys_bus.[<write_ $t>](addr, data)?;
                }
                Ok(())
            }
        }
    };
    (@read, $t:ty) => {
        paste! {
            fn [<read_ $t>](&self, addr: &u64, data: &mut $t) -> Result<(), u64> {
                if self.try_read_local(addr, data as *mut $t as *mut u8, std::mem::size_of::<$t>()).is_err() {
                    self.sys_bus.[<read_ $t>](addr, data)?;
                }
                Ok(())
            }
        }
    };
}

impl Bus for CoreBus {
    fn acquire(&self, addr: &u64, len: usize, who: usize) -> bool {
        if self.is_local(addr) {
            panic!("acquire is not supported for local memory!")
        }
        self.sys_bus.acquire(addr, len, who)
    }
    fn lock_holder(&self, addr: &u64, len: usize) -> Option<usize> {
        if self.is_local(addr) {
            return None
        }
        self.sys_bus.lock_holder(addr, len)
    }
    fn invalid_lock(&self, addr: &u64, len: usize, who: usize) {
        if self.is_local(addr) {
            panic!("invalid_lock is not supported for local memory!")
        }
        self.sys_bus.invalid_lock(addr, len, who)
    }
    fn release(&self, who: usize) {
        self.sys_bus.release(who)
    }

    corebus_add_access!(u8, u16, u32, u64);
}

#[derive_io(Bytes, U8, U16, U32, U64)]
pub struct ExtBus {
    pub name: String,
    pub id: u32,
    pub base: u64,
    pub size: u64,
}
macro_rules! extbus_add_access {
    ($($t:ty),+ ) => {
        $(
            paste! {
                impl [<$t:upper Access>] for ExtBus {
                    extbus_add_access!(@writefn, $t);
                    extbus_add_access!(@readfn, $t);
                }
            }
        )+
    };
    (@writefn, $t:ty) => {
        paste! {
            fn write(&self, addr: &u64, data: $t) {
                extern "C" {
                    fn [<cluster_ext_write_ $t>](id:u32, addr: u64, data: $t);
                }
                unsafe {
                    [<cluster_ext_write_ $t>](self.id, self.base + *addr as u64, data);
                }
            }
        }
    };
    (@readfn, $t:ty) => {
        paste! {
            fn read(&self, addr: &u64) -> $t {
                extern "C" {
                    fn [<cluster_ext_read_ $t>](id:u32, addr: u64, data: &mut $t);
                }
                unsafe {
                    let mut data:$t = 0;
                    [<cluster_ext_read_ $t>](self.id, self.base + *addr as u64, &mut data);
                    data
                }
            }
        }
    };
}

extbus_add_access!(u8, u16, u32, u64);

impl BytesAccess for ExtBus {
    fn write(&self, addr: &u64, data: &[u8]) -> Result<usize, String> {
        macro_rules! dispatch_to {
            ($t:ty) => {
                paste! {
                    {
                        let mut bytes = [0;std::mem::size_of::<$t>()];
                        bytes.copy_from_slice(data);
                        [<$t:upper Access>]::write(self, addr,$t::from_le_bytes(bytes))
                    }
                }
            };
        }
        match data.len() {
            1 => dispatch_to!(u8),
            2 => dispatch_to!(u16),
            4 => dispatch_to!(u32),
            8 => dispatch_to!(u64),
            _ => unreachable!(),
        }
        Ok(0)
    }

    fn read(&self, addr: &u64, data: &mut [u8]) -> Result<usize, String> {
        macro_rules! dispatch_to {
            ($t:ty) => {
                paste! {
                    data.copy_from_slice(&[<$t:upper Access>]::read(self, addr).to_le_bytes())
                }
            };
        }
        match data.len() {
            1 => dispatch_to!(u8),
            2 => dispatch_to!(u16),
            4 => dispatch_to!(u32),
            8 => dispatch_to!(u64),
            _ => unreachable!(),
        }
        // println!("read @{:#x}, {:#x?}", *addr, data);
        Ok(0)
    }
}
