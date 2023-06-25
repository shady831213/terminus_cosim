#!/bin/sh
set -e
TESTNAME=$1
CUR_DIR=${PWD}
cd ${CUR_DIR}/terminus_cluster
#proxychains cargo update
cargo build --release
cd ${CUR_DIR}/tb_dpi
#proxychains cargo update
cargo build --release --features="ptr32"
cd ${CUR_DIR}/vfw
#proxychains cargo update --workspace
TESTNAME=$TESTNAME RISCV_TOOLCHAIN_PREFIX=riscv64-unknown-elf- cargo build -p terminus_cosim_tests  -Zunstable-options --release --out-dir target/$TESTNAME --bin $TESTNAME --target riscv32imac-unknown-none-elf
${RISCV_TOOLCHAIN_PREFIX}objdump -D target/$TESTNAME/$TESTNAME >| target/$TESTNAME/$TESTNAME.dump
cd ${CUR_DIR}

rm -rf ${CUR_DIR}/obj_dir

verilator --cc --exe -sv -o ${CUR_DIR}/test --vpi --top-module \
    TestModule ${CUR_DIR}/testbench/verilator_main.cc ${CUR_DIR}/testbench/tb.sv \
    ${CUR_DIR}/terminus_cluster/target/release/libterminus_cluster.a ${CUR_DIR}/tb_dpi/target/release/libtb_dpi.a \
    -CFLAGS -DVERILATOR -CFLAGS -fPIC -LDFLAGS -Wl,-Bdynamic -LDFLAGS -lpthread -LDFLAGS -ldl -LDFLAGS -lm -LDFLAGS -lrt

make -C ${CUR_DIR}/obj_dir -f VTestModule.mk

MAILBOX_CFG_FILE=${CUR_DIR}/testbench/mailbox_cfg.yaml \
MEM_CFG_FILE=${CUR_DIR}/testbench/mem_cfg.yaml \
MAILBOX_FS_ROOT=${CUR_DIR}/mb_fs_root \
ELF_FILE=${CUR_DIR}/vfw/target/$TESTNAME/$TESTNAME \
./test
