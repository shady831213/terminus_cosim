use super::common::{
    DPIBankedMemHDLBuffers, DPIBankedShareMem, DPIBlackBoxShareMem, DPIDirectShareMem,
    DPIMemHDLBuffer, DPIMemHDLBuffers, DPIShareMem, InitMethod,
};
use mailbox_rs::{mb_rpcs::*, mb_std::*};
use std::cmp::min;
extern "C" {
    fn uvm_hdl_read(path: *const i8, value: *mut u8) -> isize;
    fn uvm_hdl_deposit(path: *const i8, value: *const u8) -> isize;
}
//UVM_HDL_MAX_WIDTH = 1024, bytes=128, 4value so multiple 2
const UVM_HDL_MAX_WIDTH: usize = 1024;
const UVM_HDL_MAX_BYTES: usize = UVM_HDL_MAX_WIDTH >> 3;
const UVM_BUFFER_SIZE: usize = UVM_HDL_MAX_BYTES << 1;

//int uvm_hdl_deposit/read(char *path, p_vpi_vecval value), uvm_hdl_inca.c, line596
// t_vpi_vecval, vpi_user.h, line609
// typedef struct t_vpi_vecval
// {
//     /* following fields are repeated enough times to contain vector */
//     PLI_UINT32 aval, bval;         /* bit encoding: ab: 00=0, 10=1, 11=X, 01=Z */
// } s_vpi_vecval, *p_vpi_vecval;
fn uvm_read_bits(path: &str, data: &mut [u8]) {
    unsafe {
        let c_str = std::ffi::CString::new(path).unwrap();
        let mut buffer = [0u8; UVM_BUFFER_SIZE];
        uvm_hdl_read(c_str.as_ptr(), buffer.as_mut_ptr());
        for i in (0..data.len()).step_by(4) {
            let len = min(4, data.len() - i);
            let buffer_i = i << 1;
            data[i..i + len].copy_from_slice(&buffer[buffer_i..buffer_i + len]);
        }
    }
}

fn uvm_write_bits(path: &str, data: &[u8]) {
    unsafe {
        let c_str = std::ffi::CString::new(path).unwrap();
        let mut buffer = [0u8; UVM_BUFFER_SIZE];
        for i in (0..data.len()).step_by(4) {
            let len = min(4, data.len() - i);
            let buffer_i = i << 1;
            buffer[buffer_i..buffer_i + len].copy_from_slice(&data[i..i + len]);
        }
        uvm_hdl_deposit(c_str.as_ptr(), buffer.as_ptr());
    }
}

fn uvm_read_entry(path: &str, data: &mut [u8]) {
    for i in 0..(data.len() + UVM_HDL_MAX_BYTES - 1) / UVM_HDL_MAX_BYTES {
        let start = i * UVM_HDL_MAX_BYTES;
        let end = min((i + 1) * UVM_HDL_MAX_BYTES, data.len());
        uvm_read_bits(
            &format!("{}[{}:{}]", path, (end << 3) - 1, start << 3),
            &mut data[start..end],
        );
    }
}
fn uvm_write_entry(path: &str, data: &[u8]) {
    for i in 0..(data.len() + UVM_HDL_MAX_BYTES - 1) / UVM_HDL_MAX_BYTES {
        let start = i * UVM_HDL_MAX_BYTES;
        let end = min((i + 1) * UVM_HDL_MAX_BYTES, data.len());
        uvm_write_bits(
            &format!("{}[{}:{}]", path, (end << 3) - 1, start << 3),
            &data[start..end],
        );
    }
}

impl DPIMemHDLBuffer {
    fn idx_path(&self, i: usize) -> String {
        format!("{}[{}]", self.path(), i)
    }
    pub(super) fn sync_partial(&mut self) {
        if self.head_unaligned {
            uvm_read_entry(
                &self.idx_path(self.head_idx),
                &mut self.buffer[..self.array.width],
            );
        }
        if self.tail_unaligned {
            let buffer_end = self.buffer.len();
            uvm_read_entry(
                &self.idx_path(self.tail_idx),
                &mut self.buffer[buffer_end - self.array.width..],
            );
        }
    }
    pub(super) fn flush(&self) {
        for i in self.head_idx..=self.tail_idx {
            let buffer_idx = (i - self.head_idx) * self.array.width;
            uvm_write_entry(
                &self.idx_path(i),
                &self.buffer[buffer_idx..buffer_idx + self.array.width],
            );
        }
    }
    pub(super) fn sync_all(&mut self) {
        for i in self.head_idx..=self.tail_idx {
            let buffer_idx = (i - self.head_idx) * self.array.width;
            uvm_read_entry(
                &&self.idx_path(i),
                &mut self.buffer[buffer_idx..buffer_idx + self.array.width],
            );
        }
    }
}

impl DPIBlackBoxShareMem {
    const fn offset(&self, addr: MBPtrT) -> usize {
        (addr - self.base) as usize
    }
}

impl MBShareMem for DPIBlackBoxShareMem {
    fn write(&mut self, addr: MBPtrT, data: &[u8]) -> usize {
        extern "C" {
            fn mb_bb_mem_write(name: *const i8, addr: u32, data: u8);
        }
        let c_str = std::ffi::CString::new(self.name.as_str()).unwrap();
        for (i, d) in data.iter().enumerate() {
            unsafe {
                mb_bb_mem_write(c_str.as_ptr(), self.offset(addr) as u32 + i as u32, *d);
            }
        }
        data.len()
    }
    fn read(&self, addr: MBPtrT, data: &mut [u8]) -> usize {
        extern "C" {
            fn mb_bb_mem_read(name: *const i8, addr: u32, data: *mut u8);
        }
        let c_str = std::ffi::CString::new(self.name.as_str()).unwrap();
        for (i, d) in data.iter_mut().enumerate() {
            unsafe {
                mb_bb_mem_read(
                    c_str.as_ptr(),
                    self.offset(addr) as u32 + i as u32,
                    d as *mut u8,
                );
            }
        }
        data.len()
    }
}

impl MBShareMem for DPIDirectShareMem {
    fn write(&mut self, addr: MBPtrT, data: &[u8]) -> usize {
        let addr = self.offset(addr);
        let mut buffers = DPIMemHDLBuffers::new(&self.array, addr, data.len());
        buffers.sync_partial();
        for (i, d) in data.iter().enumerate() {
            let offset = addr + i;
            let buffer = buffers.get_buffer_mut(offset);
            buffer.set_data(offset, *d);
        }
        buffers.flush();
        data.len()
    }
    fn read(&self, addr: MBPtrT, data: &mut [u8]) -> usize {
        let addr = self.offset(addr);
        let mut buffers = DPIMemHDLBuffers::new(&self.array, addr, data.len());
        buffers.sync_all();
        for (i, d) in data.iter_mut().enumerate() {
            let offset = addr as usize + i;
            let buffer = buffers.get_buffer(offset);
            *d = buffer.get_data(offset);
        }
        data.len()
    }
}

impl MBShareMem for DPIBankedShareMem {
    fn write(&mut self, addr: MBPtrT, data: &[u8]) -> usize {
        let addr = self.offset(addr);
        let mut buffers = DPIBankedMemHDLBuffers::new(self, addr, data.len());
        buffers.sync();
        for (i, d) in data.iter().enumerate() {
            let offset = addr + i;
            let array = buffers.get_array_mut(self.row(offset), self.col(offset));
            let bank_offset = self.bank_offset(offset);
            let buffer = array.get_buffer_mut(bank_offset);
            buffer.set_data(bank_offset, *d);
        }
        buffers.flush();
        data.len()
    }
    fn read(&self, addr: MBPtrT, data: &mut [u8]) -> usize {
        let addr = self.offset(addr);
        let mut buffers = DPIBankedMemHDLBuffers::new(self, addr, data.len());
        buffers.sync();
        for (i, d) in data.iter_mut().enumerate() {
            let offset = addr + i;
            let array = buffers.get_array(self.row(offset), self.col(offset));
            let bank_offset = self.bank_offset(offset);
            let buffer = array.get_buffer(bank_offset);
            *d = buffer.get_data(bank_offset);
        }
        data.len()
    }
}

impl DPIShareMem {
    pub(super) fn init(&mut self, _method: &InitMethod) {}
}
