#![no_std]
#![no_main]
extern crate terminus_cosim;
use terminus_cosim::*;
extern "C" {
    fn c_file_access_test();
}
#[export_name = "main"]
fn file_access_test() -> u32 {
    {
        let mut f = fs::open(
            "test",
            fs::HWAL_FILE_WRITE | fs::HWAL_FILE_READ | fs::HWAL_FILE_TRUNC,
        );
        f.write(b"file_access_test!\n");
        f.seek(0);
        let mut buf: [u8; 128] = [0; 128];
        f.read(&mut buf);
        cprintln!("%s", buf.as_ptr());
    }
    unsafe {
        c_file_access_test();
    }
    0
}
