use mailbox_rs::{mb_rpcs::*, mb_std::*};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Copy, Clone, PartialEq)]
struct DPIMemArrayDim {
    rows: usize,
    cols: usize,
}

impl DPIMemArrayDim {
    const fn row(&self, idx: usize) -> usize {
        idx / self.cols
    }
    const fn col(&self, idx: usize) -> usize {
        idx % self.cols
    }
    const fn depth(&self) -> usize {
        self.cols * self.rows
    }
}

#[derive(Debug, PartialEq)]
pub(super) struct DPIMemArray {
    path: String,
    dim: DPIMemArrayDim,
    pub(super) width: usize,
}

impl DPIMemArray {
    const fn new(path: String, dim: DPIMemArrayDim, width: usize) -> DPIMemArray {
        DPIMemArray { path, dim, width }
    }
    const fn size(&self) -> usize {
        self.depth() * self.width
    }
    const fn idx_array(&self, offset: usize) -> usize {
        offset / self.width
    }
    const fn idx_byte(&self, offset: usize) -> usize {
        offset & (self.width.next_power_of_two() - 1)
    }
    const fn depth(&self) -> usize {
        self.dim.depth()
    }
    const fn row(&self, offset: usize) -> usize {
        self.dim.row(self.idx_array(offset))
    }
    const fn col(&self, offset: usize) -> usize {
        self.dim.col(self.idx_array(offset))
    }
    fn array_hdl_path(&self, row: usize) -> String {
        if self.dim.rows > 1 {
            format!("{}[{}]", self.path, row)
        } else {
            self.path.clone()
        }
    }
}

pub(super) struct DPIMemHDLBuffer {
    pub(super) row: usize,
    pub(super) head_idx: usize,
    pub(super) tail_idx: usize,
    pub(super) head_unaligned: bool,
    pub(super) tail_unaligned: bool,
    pub(super) array: Arc<DPIMemArray>,
    pub(super) buffer: Vec<u8>,
}

impl DPIMemHDLBuffer {
    pub(super) fn get_data(&self, offset: usize) -> u8 {
        self.buffer[(self.array.col(offset) - self.head_idx) * self.array.width
            + self.array.idx_byte(offset)]
    }

    pub(super) fn set_data(&mut self, offset: usize, data: u8) {
        self.buffer[(self.array.col(offset) - self.head_idx) * self.array.width
            + self.array.idx_byte(offset)] = data;
    }

    pub(super) fn path(&self) -> String {
        self.array.array_hdl_path(self.row)
    }
}

pub(super) struct DPIMemHDLBuffers {
    pub(super) start_idx: usize,
    pub(super) array: Arc<DPIMemArray>,
    pub(super) buffers: Vec<DPIMemHDLBuffer>,
}

impl DPIMemHDLBuffers {
    pub(super) fn new(array: &Arc<DPIMemArray>, offset: usize, len: usize) -> DPIMemHDLBuffers {
        let offset_end = offset + len - 1;
        let start_row = array.row(offset);
        let end_row = array.row(offset_end);
        let mut buffers = vec![];
        for row in start_row..=end_row {
            let head_idx = if row == start_row {
                array.col(offset)
            } else {
                0
            };
            let head_unaligned = if row == start_row {
                (offset & (array.width.next_power_of_two() - 1)) != 0
            } else {
                false
            };
            let tail_idx = if row == end_row {
                array.col(offset_end)
            } else {
                array.dim.cols - 1
            };
            let tail_unaligned = if row == end_row {
                ((offset_end & (array.width.next_power_of_two() - 1))
                    != (array.width.next_power_of_two() - 1))
                    && !(head_unaligned && head_idx == tail_idx)
            } else {
                false
            };
            buffers.push(DPIMemHDLBuffer {
                row,
                head_idx,
                tail_idx,
                head_unaligned,
                tail_unaligned,
                array: array.clone(),
                buffer: vec![0u8; (tail_idx - head_idx + 1) * array.width],
            });
        }
        DPIMemHDLBuffers {
            start_idx: start_row,
            array: array.clone(),
            buffers,
        }
    }

    pub(super) fn get_buffer(&self, offset: usize) -> &DPIMemHDLBuffer {
        &self.buffers[self.array.row(offset) - self.start_idx]
    }

    pub(super) fn get_buffer_mut(&mut self, offset: usize) -> &mut DPIMemHDLBuffer {
        &mut self.buffers[self.array.row(offset) - self.start_idx]
    }

    pub(super) fn sync_partial(&mut self) {
        for b in self.buffers.iter_mut() {
            b.sync_partial()
        }
    }
    pub(super) fn sync_all(&mut self) {
        for b in self.buffers.iter_mut() {
            b.sync_all()
        }
    }
    pub(super) fn flush(&self) {
        for b in self.buffers.iter() {
            b.flush()
        }
    }
}

impl Deref for DPIMemHDLBuffers {
    type Target = Vec<DPIMemHDLBuffer>;
    fn deref(&self) -> &Self::Target {
        &self.buffers
    }
}

impl DerefMut for DPIMemHDLBuffers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffers
    }
}

#[derive(Debug, PartialEq)]
pub enum DPIShareMem {
    BlackBox(DPIBlackBoxShareMem),
    Direct(DPIDirectShareMem),
    Banked(DPIBankedShareMem),
}

impl DPIShareMem {
    fn check(self) -> Result<DPIShareMem, String> {
        match &self {
            DPIShareMem::BlackBox(_) => {}
            DPIShareMem::Direct(mem) => mem.check()?,
            DPIShareMem::Banked(mem) => mem.check()?,
        }
        Ok(self)
    }
}

impl MBShareMemBlock for DPIShareMem {
    fn base(&self) -> MBPtrT {
        match &self {
            DPIShareMem::BlackBox(mem) => mem.base(),
            DPIShareMem::Direct(mem) => mem.base(),
            DPIShareMem::Banked(mem) => mem.base(),
        }
    }
    fn size(&self) -> MBPtrT {
        match &self {
            DPIShareMem::BlackBox(mem) => mem.size(),
            DPIShareMem::Direct(mem) => mem.size(),
            DPIShareMem::Banked(mem) => mem.size(),
        }
    }
}

impl MBShareMem for DPIShareMem {
    fn write(&mut self, addr: MBPtrT, data: &[u8]) -> usize {
        if data.len() == 0 {
            return 0;
        }
        let len = if self.in_range((addr as usize + data.len() - 1) as MBPtrT) {
            data.len()
        } else {
            (self.end_addr() - addr + 1) as usize
        };
        match self {
            DPIShareMem::BlackBox(mem) => mem.write(addr, &data[..len]),
            DPIShareMem::Direct(mem) => mem.write(addr, &data[..len]),
            DPIShareMem::Banked(mem) => mem.write(addr, &data[..len]),
        }
    }
    fn read(&self, addr: MBPtrT, data: &mut [u8]) -> usize {
        if data.len() == 0 {
            return 0;
        }
        let len = if self.in_range((addr as usize + data.len() - 1) as MBPtrT) {
            data.len()
        } else {
            (self.end_addr() - addr + 1) as usize
        };
        match self {
            DPIShareMem::BlackBox(mem) => mem.read(addr, &mut data[..len]),
            DPIShareMem::Direct(mem) => mem.read(addr, &mut data[..len]),
            DPIShareMem::Banked(mem) => mem.read(addr, &mut data[..len]),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DPIBlackBoxShareMem {
    pub(super) name: String,
    pub(super) base: MBPtrT,
    size: usize,
}

impl DPIBlackBoxShareMem {
    fn new(name: String, base: MBPtrT, size: usize) -> DPIBlackBoxShareMem {
        DPIBlackBoxShareMem { name, base, size }
    }
}

impl MBShareMemBlock for DPIBlackBoxShareMem {
    fn base(&self) -> MBPtrT {
        self.base
    }
    fn size(&self) -> MBPtrT {
        self.size as MBPtrT
    }
}

#[derive(Debug, PartialEq)]
pub struct DPIDirectShareMem {
    pub(super) name: String,
    base: MBPtrT,
    size: usize,
    pub(super) array: Arc<DPIMemArray>,
}

impl DPIDirectShareMem {
    fn new(
        name: String,
        path: String,
        width: usize,
        base: MBPtrT,
        size: usize,
        array: Option<DPIMemArrayDim>,
    ) -> DPIDirectShareMem {
        DPIDirectShareMem {
            name,
            base,
            size,
            array: Arc::new(DPIMemArray::new(
                path,
                if let Some(dim) = array {
                    dim
                } else {
                    DPIMemArrayDim {
                        rows: 1,
                        cols: size / (width >> 3),
                    }
                },
                width >> 3,
            )),
        }
    }
    pub(super) const fn offset(&self, addr: MBPtrT) -> usize {
        (addr - self.base) as usize
    }

    fn check(&self) -> Result<(), String> {
        if self.size != self.array.size() {
            Err(format!(
                "{} error: array dims {:?} with byte width {} is mismatched with size {}",
                self.name, self.array.dim, self.array.width, self.size
            ))
        } else {
            Ok(())
        }
    }
}

impl MBShareMemBlock for DPIDirectShareMem {
    fn base(&self) -> MBPtrT {
        self.base
    }
    fn size(&self) -> MBPtrT {
        self.size as MBPtrT
    }
}

#[derive(Debug, PartialEq)]
pub struct DPIBankedShareMem {
    pub(super) name: String,
    path: String,
    width: usize,
    base: MBPtrT,
    size: usize,
    bank_width: usize,
    bank_depth: usize,
    banks: Vec<Vec<Arc<DPIMemArray>>>,
}

impl DPIBankedShareMem {
    fn new(
        name: String,
        path: String,
        width: usize,
        base: MBPtrT,
        size: usize,
        bank_width: usize,
        bank_depth: usize,
        banks: Vec<Vec<String>>,
        array: Option<DPIMemArrayDim>,
    ) -> DPIBankedShareMem {
        let array = if let Some(dim) = array {
            dim
        } else {
            DPIMemArrayDim {
                rows: 1,
                cols: bank_depth,
            }
        };
        let width = width >> 3;
        let bank_width = bank_width >> 3;
        DPIBankedShareMem {
            name,
            path: path.clone(),
            width,
            base,
            size,
            bank_width,
            bank_depth,
            banks: banks
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|col| {
                            Arc::new(DPIMemArray::new(
                                format!("{}.{}", &path, col),
                                array,
                                bank_width,
                            ))
                        })
                        .collect()
                })
                .collect(),
        }
    }
    const fn depth(&self) -> usize {
        self.size / self.width
    }
    const fn cols(&self) -> usize {
        self.width / self.bank_width
    }
    const fn rows(&self) -> usize {
        self.depth() / self.bank_depth
    }
    pub(super) const fn row(&self, offset: usize) -> usize {
        (offset / (self.width * self.bank_depth)) & (self.rows().next_power_of_two() - 1)
    }
    pub(super) const fn col(&self, offset: usize) -> usize {
        (offset / self.bank_width) & (self.cols().next_power_of_two() - 1)
    }
    pub(super) const fn offset(&self, addr: MBPtrT) -> usize {
        (addr - self.base) as usize
    }
    pub(super) const fn bank_offset(&self, offset: usize) -> usize {
        ((offset / self.width) % self.bank_depth)
            << self.bank_width.next_power_of_two().trailing_zeros()
            | offset & (self.bank_width.next_power_of_two() - 1)
    }
    fn check(&self) -> Result<(), String> {
        if self.rows() != self.banks.len() {
            return Err(format!("mem {} error: rows are mismatched, banks rows = {}, size = {}, width = {}, bank_depth = {}, bank_width = {}, expect rows = {}!",
            self.name, self.banks.len(), self.size, self.width, self.bank_depth, self.bank_width, self.rows()));
        } else {
            for (i, row) in self.banks.iter().enumerate() {
                if self.cols() != row.len() {
                    return Err(format!("mem {} error: cols of row[{}] are mismatched, bank col = {}, size = {}, width = {}, bank_depth = {}, bank_width = {}, expect cols = {}!",
                    self.name, i, row.len(), self.size, self.width, self.bank_depth, self.bank_width, self.cols()));
                }
            }
        }
        if self.bank_depth != self.banks[0][0].depth() {
            return Err(format!(
                "{} error: array dims {:?} is mismatched with bank_depth {}",
                self.name, self.banks[0][0].dim, self.bank_depth
            ));
        }
        Ok(())
    }
}

impl MBShareMemBlock for DPIBankedShareMem {
    fn base(&self) -> MBPtrT {
        self.base
    }
    fn size(&self) -> MBPtrT {
        self.size as MBPtrT
    }
}

pub(super) struct DPIBankedMemHDLBuffers {
    start_row: usize,
    buffers: Vec<Vec<DPIMemHDLBuffers>>,
}

impl DPIBankedMemHDLBuffers {
    pub(super) fn new(
        mem: &DPIBankedShareMem,
        offset: usize,
        len: usize,
    ) -> DPIBankedMemHDLBuffers {
        let offset_end = offset + len - 1;
        let start_row = mem.row(offset);
        let end_row = mem.row(offset_end);
        let row_buffers: Vec<Vec<DPIMemHDLBuffers>> = mem.banks[start_row..=end_row]
            .iter()
            .enumerate()
            .map(|(r, row)| {
                let start_bank_col = if r == 0 { mem.bank_offset(offset) } else { 0 };
                let end_bank_col = if r == end_row - start_row {
                    mem.bank_offset(offset_end)
                } else {
                    mem.bank_depth * mem.bank_width - 1
                };
                let col_len = end_bank_col - start_bank_col + 1;
                row.iter()
                    .map(|col| DPIMemHDLBuffers::new(col, start_bank_col, col_len))
                    .collect()
            })
            .collect();
        DPIBankedMemHDLBuffers {
            start_row,
            buffers: row_buffers,
        }
    }
    pub(super) fn get_array(&self, row: usize, col: usize) -> &DPIMemHDLBuffers {
        &self.buffers[row - self.start_row][col]
    }
    pub(super) fn get_array_mut(&mut self, row: usize, col: usize) -> &mut DPIMemHDLBuffers {
        &mut self.buffers[row - self.start_row][col]
    }
    pub(super) fn sync(&mut self) {
        for r in self.buffers.iter_mut() {
            for c in r.iter_mut() {
                c.sync_all();
            }
        }
    }
    pub(super) fn flush(&self) {
        for r in self.buffers.iter() {
            for c in r.iter() {
                c.flush();
            }
        }
    }
}

impl Deref for DPIBankedMemHDLBuffers {
    type Target = Vec<Vec<DPIMemHDLBuffers>>;
    fn deref(&self) -> &Self::Target {
        &self.buffers
    }
}

impl DerefMut for DPIBankedMemHDLBuffers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffers
    }
}

#[derive(Debug)]
pub(super) enum InitMethod {
    NoInit,
    AllZero,
    Random,
    Hex(PathBuf),
}

impl std::convert::TryFrom<&Yaml> for InitMethod {
    type Error = String;
    fn try_from(doc: &Yaml) -> Result<Self, Self::Error> {
        if let Yaml::BadValue = doc {
            Ok(InitMethod::NoInit)
        } else {
            let keys = ["no_init", "all_zero", "random", "hex"];
            let method = get_yaml_with_ref(doc, "method")
                .as_str()
                .ok_or(format!("method should be in {:?}!", &keys))?;
            if !keys.contains(&method) {
                return Err(format!("method should be in {:?}!", &keys));
            }
            match method {
                "no_init" => Ok(InitMethod::NoInit),
                "all_zero" => Ok(InitMethod::AllZero),
                "random" => Ok(InitMethod::Random),
                "hex" => {
                    let file = get_yaml_with_ref(doc, "file").as_str().ok_or(
                        "if init method is hex, file is required and should be path!".to_string(),
                    )?;
                    let file = PathBuf::from(
                        shellexpand::full(file)
                            .map_err(|e| e.to_string())?
                            .to_string()
                            .as_str(),
                    );
                    if !file.is_file() {
                        Err(format!("hex file {} does not exist!", file.display()))
                    } else {
                        Ok(InitMethod::Hex(file))
                    }
                }
                _ => Err(format!("method should be in {:?}!", &keys)),
            }
        }
    }
}

#[derive(Default)]
pub struct DPIShareMemParser;
impl MBShareMemParser for DPIShareMemParser {
    type MemType = DPIShareMem;
    fn parse(&self, key: &str, doc: &Yaml) -> Result<Self::MemType, String> {
        let array = {
            let array_dims = get_yaml_with_ref(doc, "array_dims");
            match array_dims {
                Yaml::BadValue => None,
                _ => Some(DPIMemArrayDim {
                    rows: get_yaml_with_ref(array_dims, "rows")
                        .as_i64()
                        .ok_or(format!("{} error: rows should be integer!", key))?
                        as usize,
                    cols: get_yaml_with_ref(array_dims, "cols")
                        .as_i64()
                        .ok_or(format!("{} error: cols should be integer!", key))?
                        as usize,
                }),
            }
        };
        let base = get_yaml_with_ref(doc, "base")
            .as_i64()
            .ok_or(format!("{} error: base should be integer!", key))? as MBPtrT;
        let size = get_yaml_with_ref(doc, "size")
            .as_i64()
            .ok_or(format!("{} error: size should be integer!", key))? as usize;
        let init_method = InitMethod::try_from(get_yaml_with_ref(doc, "init"))
            .map_err(|e| format!("{} error: {}", key, e))?;
        let mut mem = if let Some(path) = get_yaml_with_ref(doc, "path").as_str() {
            let path = path.to_string();
            let width = get_yaml_with_ref(doc, "width")
                .as_i64()
                .ok_or(format!("{} error: width should be integer!", key))?
                as usize;
            if let Some(banks) = get_yaml_with_ref(doc, "banks").as_vec() {
                let banks = banks
                    .iter()
                    .map(|row| -> Result<Vec<String>, String> {
                        let row = row.as_vec().ok_or(format!(
                            "{} error: banks should be 2d array of string!",
                            key
                        ))?;
                        row.iter()
                            .map(|e| {
                                Ok(e.as_str()
                                    .ok_or(format!(
                                        "{} error: banks should be 2d array of string!",
                                        key
                                    ))?
                                    .to_string())
                            })
                            .collect()
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                let bank_width = get_yaml_with_ref(doc, "bank_width")
                    .as_i64()
                    .ok_or(format!("{} error: bank_width should be integer!", key))?
                    as usize;
                let bank_depth = get_yaml_with_ref(doc, "bank_depth")
                    .as_i64()
                    .ok_or(format!("{} error: bank_depth should be integer!", key))?
                    as usize;
                DPIShareMem::Banked(DPIBankedShareMem::new(
                    key.to_string(),
                    path,
                    width,
                    base,
                    size,
                    bank_width,
                    bank_depth,
                    banks,
                    array,
                ))
                .check()
            } else {
                DPIShareMem::Direct(DPIDirectShareMem::new(
                    key.to_string(),
                    path,
                    width,
                    base,
                    size,
                    array,
                ))
                .check()
            }
        } else {
            DPIShareMem::BlackBox(DPIBlackBoxShareMem::new(key.to_string(), base, size)).check()
        }?;
        mem.init(&init_method);
        Ok(mem)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl DPIMemHDLBuffer {
        pub fn sync_partial(&mut self) {
            if self.head_unaligned {
                println!("{} sync {} to buffer [0:width]", self.path(), self.head_idx)
            }
            if self.tail_unaligned {
                println!(
                    "{} sync {} to buffer [end-width:end]",
                    self.path(),
                    self.tail_idx
                )
            }
        }
        pub fn flush(&self) {
            println!(
                "{} write {} data to [{}:{}]",
                self.path(),
                self.buffer.len(),
                self.head_idx,
                self.tail_idx
            )
        }
        pub fn sync_all(&mut self) {
            println!(
                "{} sync [{}:{}] to buffer",
                self.path(),
                self.head_idx,
                self.tail_idx
            )
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

    impl MBShareMem for DPIBlackBoxShareMem {
        fn write(&mut self, _addr: MBPtrT, data: &[u8]) -> usize {
            data.len()
        }
        fn read(&self, _addr: MBPtrT, data: &mut [u8]) -> usize {
            data.len()
        }
    }

    impl DPIShareMem {
        pub(super) fn init(&mut self, _method: &InitMethod) {}
    }

    #[test]
    fn direct_mem_test() {
        let mem = "
radio_cim0:
    path: Pc805Tb.dut.IPc805.core_0.cpu_0.radio_0.imem_0.sram_0.ram.mem_model_0.memory
    base: 0x05000000
    width: 128
    size: 65536
        ";
        let docs = YamlLoader::load_from_str(mem).unwrap();
        let (name, v) = docs[0].as_hash().unwrap().front().unwrap();
        let mut mem = DPIShareMemParser.parse(name.as_str().unwrap(), &v).unwrap();
        println!("mem:{:#x?}", &mem);
        assert_eq!(mem,
            DPIShareMem::Direct(
                DPIDirectShareMem {
                    name: "radio_cim0".to_string(),
                    base: 0x5000000,
                    size: 0x10000,
                    array: Arc::new(DPIMemArray {
                        path: "Pc805Tb.dut.IPc805.core_0.cpu_0.radio_0.imem_0.sram_0.ram.mem_model_0.memory".to_string(),
                        dim: DPIMemArrayDim {
                            rows: 0x1,
                            cols: 0x1000,
                        },
                        width: 16,
                    }),
                }
            )
        );
        let mut data = [5u8; 10];
        assert_eq!(10, mem.write(0x5000000, &data));
        assert_eq!(10, mem.write(0x5000005, &data));
        assert_eq!(10, mem.write(0x500000a, &data));
        assert_eq!(10, mem.read(0x5000000, &mut data));
        assert_eq!(10, mem.read(0x5000005, &mut data));
        assert_eq!(10, mem.read(0x500000a, &mut data));
    }

    #[test]
    fn direct_mem_with_array_test() {
        let mem = "
radio_cim0:
    path: Pc805Tb.dut.IPc805.core_0.cpu_0.radio_0.imem_0.sram_0.ram.mem_model_0.memory
    base: 0x05000000
    width: 128
    size: 65536
    array_dims:
        rows: 256
        cols: 16
        ";
        let docs = YamlLoader::load_from_str(mem).unwrap();
        let (name, v) = docs[0].as_hash().unwrap().front().unwrap();
        let mut mem = DPIShareMemParser.parse(name.as_str().unwrap(), &v).unwrap();
        println!("mem:{:#x?}", &mem);
        assert_eq!(mem,
            DPIShareMem::Direct(
                DPIDirectShareMem {
                    name: "radio_cim0".to_string(),
                    base: 0x5000000,
                    size: 0x10000,
                    array: Arc::new(DPIMemArray {
                        path: "Pc805Tb.dut.IPc805.core_0.cpu_0.radio_0.imem_0.sram_0.ram.mem_model_0.memory".to_string(),
                        dim: DPIMemArrayDim {
                            rows: 256,
                            cols: 16,
                        },
                        width: 16,
                    }),
                }
            )
        );
        let mut data = [5u8; 10];
        assert_eq!(10, mem.write(0x5000000, &data));
        assert_eq!(10, mem.write(0x5000005, &data));
        assert_eq!(10, mem.write(0x500000a, &data));
        assert_eq!(10, mem.read(0x5000000, &mut data));
        assert_eq!(10, mem.read(0x5000005, &mut data));
        assert_eq!(10, mem.read(0x500000a, &mut data));

        assert_eq!(10, mem.write(0x5000100, &data));
        assert_eq!(10, mem.write(0x5000105, &data));
        assert_eq!(10, mem.write(0x50000fa, &data));
        assert_eq!(10, mem.read(0x5000100, &mut data));
        assert_eq!(10, mem.read(0x5000105, &mut data));
        assert_eq!(10, mem.read(0x50000fa, &mut data));

        let mut data = [5u8; 10 + 256];
        assert_eq!(10 + 256, mem.write(0x50000fa, &data));
        assert_eq!(10 + 256, mem.read(0x50000fa, &mut data));
    }

    #[test]
    fn banked_mem1d_vet_test() {
        let mem = "
radio_cim0:
    path: Pc805Tb.dut.IPc805.core_0.cpu_0.radio_0.imem_0.sram_0
    base: 0x05000000
    width: 128
    size: 262144
    bank_width: 128
    bank_depth: 4096
    banks:
      - - gen_rows[0].ram.MEMORY
      - - gen_rows[1].ram.MEMORY
      - - gen_rows[2].ram.MEMORY
      - - gen_rows[3].ram.MEMORY
        ";
        let docs = YamlLoader::load_from_str(mem).unwrap();
        let (name, v) = docs[0].as_hash().unwrap().front().unwrap();
        let mut mem = DPIShareMemParser.parse(name.as_str().unwrap(), &v).unwrap();
        println!("mem:{:#x?}", &mem);
        assert_eq!(mem,
            DPIShareMem::Banked(
                DPIBankedShareMem {
                    name: "radio_cim0".to_string(),
                    path: "Pc805Tb.dut.IPc805.core_0.cpu_0.radio_0.imem_0.sram_0".to_string(),
                    width: 16,
                    base: 0x5000000,
                    size: 0x40000,
                    bank_width: 16,
                    bank_depth: 0x1000,
                    banks: vec![
                        vec![
                            Arc::new(DPIMemArray {
                                path: "Pc805Tb.dut.IPc805.core_0.cpu_0.radio_0.imem_0.sram_0.gen_rows[0].ram.MEMORY".to_string(),
                                dim: DPIMemArrayDim {
                                    rows: 0x1,
                                    cols: 0x1000,
                                },
                                width: 16,
                            }),
                        ],
                        vec![
                            Arc::new(DPIMemArray {
                                path: "Pc805Tb.dut.IPc805.core_0.cpu_0.radio_0.imem_0.sram_0.gen_rows[1].ram.MEMORY".to_string(),
                                dim: DPIMemArrayDim {
                                    rows: 0x1,
                                    cols: 0x1000,
                                },
                                width: 16,
                            }),
                        ],
                        vec![
                            Arc::new(DPIMemArray {
                                path: "Pc805Tb.dut.IPc805.core_0.cpu_0.radio_0.imem_0.sram_0.gen_rows[2].ram.MEMORY".to_string(),
                                dim: DPIMemArrayDim {
                                    rows: 0x1,
                                    cols: 0x1000,
                                },
                                width: 16,
                            }),
                        ],
                        vec![
                            Arc::new(DPIMemArray {
                                path: "Pc805Tb.dut.IPc805.core_0.cpu_0.radio_0.imem_0.sram_0.gen_rows[3].ram.MEMORY".to_string(),
                                dim: DPIMemArrayDim {
                                    rows: 0x1,
                                    cols: 0x1000,
                                },
                                width: 16,
                            }),
                        ],
                    ],
                },
            )
        );
        let mut data = [5u8; 10];
        assert_eq!(10, mem.write(0x5000000, &data));
        assert_eq!(10, mem.write(0x5000005, &data));
        assert_eq!(10, mem.write(0x500000a, &data));
        assert_eq!(10, mem.read(0x5000000, &mut data));
        assert_eq!(10, mem.read(0x5000005, &mut data));
        assert_eq!(10, mem.read(0x500000a, &mut data));
    }

    #[test]
    fn banked_mem1d_hor_test() {
        let mem = "
radio_cim0:
    path: Pc805Tb.dut.IPc805.core_0.cpu_0.radio_0.imem_0.sram_0
    base: 0x05000000
    width: 128
    size: 65536
    bank_width: 32
    bank_depth: 4096
    banks:
      - - gen_rows[0].ram.MEMORY
        - gen_rows[1].ram.MEMORY
        - gen_rows[2].ram.MEMORY
        - gen_rows[3].ram.MEMORY
        ";
        let docs = YamlLoader::load_from_str(mem).unwrap();
        let (name, v) = docs[0].as_hash().unwrap().front().unwrap();
        let mut mem = DPIShareMemParser.parse(name.as_str().unwrap(), &v).unwrap();
        println!("mem:{:#x?}", &mem);
        let mut data = [5u8; 10];
        assert_eq!(10, mem.write(0x5000000, &data));
        assert_eq!(10, mem.write(0x5000005, &data));
        assert_eq!(10, mem.write(0x500000a, &data));
        assert_eq!(10, mem.read(0x5000000, &mut data));
        assert_eq!(10, mem.read(0x5000005, &mut data));
        assert_eq!(10, mem.read(0x500000a, &mut data));
    }

    #[test]
    fn banked_mem1d_vet_array_test() {
        let mem = "
radio_cim0:
    path: Pc805Tb.dut.IPc805.core_0.cpu_0.radio_0.imem_0.sram_0
    base: 0x05000000
    width: 128
    size: 65536
    bank_width: 128
    bank_depth: 1024
    array_dims:
        rows: 64
        cols: 16
    banks:
      - - gen_rows[0].ram.MEMORY
      - - gen_rows[1].ram.MEMORY
      - - gen_rows[2].ram.MEMORY
      - - gen_rows[3].ram.MEMORY
        ";
        let docs = YamlLoader::load_from_str(mem).unwrap();
        let (name, v) = docs[0].as_hash().unwrap().front().unwrap();
        let mut mem = DPIShareMemParser.parse(name.as_str().unwrap(), &v).unwrap();
        println!("mem:{:#x?}", &mem);
        let mut data = [5u8; 10];
        assert_eq!(10, mem.write(0x5000000, &data));
        assert_eq!(10, mem.write(0x5000005, &data));
        assert_eq!(10, mem.write(0x500000a, &data));
        assert_eq!(10, mem.read(0x5000000, &mut data));
        assert_eq!(10, mem.read(0x5000005, &mut data));
        assert_eq!(10, mem.read(0x500000a, &mut data));

        assert_eq!(10, mem.write(0x5000100, &data));
        assert_eq!(10, mem.write(0x5000105, &data));
        assert_eq!(10, mem.write(0x50000fa, &data));
        assert_eq!(10, mem.read(0x5000100, &mut data));
        assert_eq!(10, mem.read(0x5000105, &mut data));
        assert_eq!(10, mem.read(0x50000fa, &mut data));

        println!("test cross 2d array row");
        let mut data = [5u8; 10 + 256];
        assert_eq!(10 + 256, mem.write(0x50000fa, &data));
        assert_eq!(10 + 256, mem.read(0x50000fa, &mut data));

        assert_eq!(10 + 256, mem.write(0x5003ffa, &data));
        assert_eq!(10 + 256, mem.read(0x5003ffa, &mut data));

        println!("test cross bank row");
        let mut data = [5u8; 10 + 16 * 1024];
        assert_eq!(10 + 16 * 1024, mem.write(0x50000fa, &data));
        assert_eq!(10 + 16 * 1024, mem.read(0x50000fa, &mut data));

        assert_eq!(10 + 16 * 1024, mem.write(0x5003ffa, &data));
        assert_eq!(10 + 16 * 1024, mem.read(0x5003ffa, &mut data));
    }

    #[test]
    fn banked_mem1d_hor_array_test() {
        let mem = "
radio_cim0:
    path: Pc805Tb.dut.IPc805.core_0.cpu_0.radio_0.imem_0.sram_0
    base: 0x05000000
    width: 128
    size: 65536
    bank_width: 32
    bank_depth: 4096
    array_dims:
        rows: 64
        cols: 64
    banks:
      - - gen_rows[0].ram.MEMORY
        - gen_rows[1].ram.MEMORY
        - gen_rows[2].ram.MEMORY
        - gen_rows[3].ram.MEMORY
        ";
        let docs = YamlLoader::load_from_str(mem).unwrap();
        let (name, v) = docs[0].as_hash().unwrap().front().unwrap();
        let mut mem = DPIShareMemParser.parse(name.as_str().unwrap(), &v).unwrap();
        println!("mem:{:#x?}", &mem);
        let mut data = [5u8; 10];
        assert_eq!(10, mem.write(0x5000000, &data));
        assert_eq!(10, mem.write(0x5000005, &data));
        assert_eq!(10, mem.write(0x500000a, &data));
        assert_eq!(10, mem.read(0x5000000, &mut data));
        assert_eq!(10, mem.read(0x5000005, &mut data));
        assert_eq!(10, mem.read(0x500000a, &mut data));

        assert_eq!(10, mem.write(0x5000100, &data));
        assert_eq!(10, mem.write(0x5000105, &data));
        assert_eq!(10, mem.write(0x50000fa, &data));
        assert_eq!(10, mem.read(0x5000100, &mut data));
        assert_eq!(10, mem.read(0x5000105, &mut data));
        assert_eq!(10, mem.read(0x50000fa, &mut data));

        println!("test cross 2d array row");
        let mut data = [5u8; 10 + 256];
        assert_eq!(10 + 256, mem.write(0x50000fa, &data));
        assert_eq!(10 + 256, mem.read(0x50000fa, &mut data));

        assert_eq!(10 + 256, mem.write(0x5003ffa, &data));
        assert_eq!(10 + 256, mem.read(0x5003ffa, &mut data));

        println!("test cross bank row");
        let mut data = [5u8; 10 + 16 * 1024];
        assert_eq!(10 + 16 * 1024, mem.write(0x50000fa, &data));
        assert_eq!(10 + 16 * 1024, mem.read(0x50000fa, &mut data));

        assert_eq!(10 + 16 * 1024, mem.write(0x5003ffa, &data));
        assert_eq!(10 + 16 * 1024, mem.read(0x5003ffa, &mut data));
    }

    #[test]
    fn banked_mem2d_hor_test() {
        let mem = "
radio_cim0:
    path: Pc805Tb.dut.IPc805.core_0.cpu_0.radio_0.imem_0.sram_0
    base: 0x05000000
    width: 128
    size: 65536
    bank_width: 64
    bank_depth: 1024
    array_dims:
        rows: 128
        cols: 8
    banks:
      - - gen_rows[0].gen_col[0].ram.MEMORY
        - gen_rows[0].gen_col[1].ram.MEMORY
      - - gen_rows[1].gen_col[0].ram.MEMORY
        - gen_rows[1].gen_col[1].ram.MEMORY
      - - gen_rows[2].gen_col[0].ram.MEMORY
        - gen_rows[2].gen_col[1].ram.MEMORY
      - - gen_rows[3].gen_col[0].ram.MEMORY
        - gen_rows[3].gen_col[1].ram.MEMORY
        ";
        let docs = YamlLoader::load_from_str(mem).unwrap();
        let (name, v) = docs[0].as_hash().unwrap().front().unwrap();
        let mut mem = DPIShareMemParser.parse(name.as_str().unwrap(), &v).unwrap();
        println!("mem:{:#x?}", &mem);
        let mut data = [5u8; 10];
        assert_eq!(10, mem.write(0x5000000, &data));
        assert_eq!(10, mem.write(0x5000005, &data));
        assert_eq!(10, mem.write(0x500000a, &data));
        assert_eq!(10, mem.read(0x5000000, &mut data));
        assert_eq!(10, mem.read(0x5000005, &mut data));
        assert_eq!(10, mem.read(0x500000a, &mut data));

        assert_eq!(10, mem.write(0x5000100, &data));
        assert_eq!(10, mem.write(0x5000105, &data));
        assert_eq!(10, mem.write(0x50000fa, &data));
        assert_eq!(10, mem.read(0x5000100, &mut data));
        assert_eq!(10, mem.read(0x5000105, &mut data));
        assert_eq!(10, mem.read(0x50000fa, &mut data));

        println!("test cross 2d array row");
        let mut data = [5u8; 10 + 256];
        assert_eq!(10 + 256, mem.write(0x50000fa, &data));
        assert_eq!(10 + 256, mem.read(0x50000fa, &mut data));

        assert_eq!(10 + 256, mem.write(0x5003ffa, &data));
        assert_eq!(10 + 256, mem.read(0x5003ffa, &mut data));

        println!("test cross bank row");
        let mut data = [5u8; 10 + 16 * 1024];
        assert_eq!(10 + 16 * 1024, mem.write(0x50000fa, &data));
        assert_eq!(10 + 16 * 1024, mem.read(0x50000fa, &mut data));

        assert_eq!(10 + 16 * 1024, mem.write(0x5003ffa, &data));
        assert_eq!(10 + 16 * 1024, mem.read(0x5003ffa, &mut data));
    }
}
