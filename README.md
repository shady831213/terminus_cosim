# terminus_cosim
This project is a cosim sulotion of ISA + verilog. It shows [terminus](https://github.com/shady831213/terminus) is friendly to be integrated into HDL simulation enviroment. It also provides framework to support communication between cpu cores, ISA or real RTL cpu cores, and the host machine though [mailbox_rs](https://github.com/shady831213/mailbox_rs) and [vfw_rs](https://github.com/shady831213/vfw_rs).Besides,  [vfw_rs](https://github.com/shady831213/vfw_rs) provides testcase framework with some useful services, and it supports both [rust testcases](https://github.com/shady831213/vfw_rs/tree/master/platform/terminus_cosim/terminus_cosim_tests/src/bin/hello_world) and [c testcases](https://github.com/shady831213/vfw_rs/tree/master/platform/terminus_cosim/terminus_cosim_tests/src/bin/hello_world_c).

## Main Dependencies
  - HDL Simulator: [verilator](https://www.veripool.org/wiki/verilator).
  - Toolchain: 
      + Rust nightly toolchain with target riscv32imac-unknown-none-elf
      + [bindgen](https://github.com/rust-lang/rust-bindgen) [dependencis](https://github.com/KyleMayes/clang-sys#environment-variables)
      + [riscv-gnu-toolchain](https://github.com/riscv/riscv-gnu-toolchain) if you want C testcase supporting.
      + [cargo-binutils](https://github.com/rust-embedded/cargo-binutils)


## Getting Start
```
// Suppose all dependencies are ready
git clone https://github.com/shady831213/terminus_cosim
cd terminus_cosim
git submodule update --init
./run.sh [hello_world|hello_world_c|trap|wait_event]
```

Then you should get:
![](https://github.com/shady831213/terminus_cosim/blob/master/hello_world.PNG)


## About other EDA tools
Other EDA tools, such as xrun, are also supported, but you need modify some DPI method to adapt to them.
