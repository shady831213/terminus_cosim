#![allow(dead_code)]
extern crate paste;
extern crate terminus;
use std::rc::Rc;
use terminus::devices::bus::TerminusBus;
use terminus::devices::clint::*;
use terminus::global::*;
use terminus::memory::{region::*, MemInfo};
use terminus::processor::Processor;
use terminus::processor::ProcessorCfg;

mod bus;
use bus::{CoreBus, ExtBus};

struct Cluster {
    processors: Vec<Processor>,
    sys_bus: Option<Rc<TerminusBus>>,
}

static mut CLUSTER: Cluster = Cluster {
    processors: vec![],
    sys_bus: None,
};

#[no_mangle]
extern "C" fn cluster_init(num_cores: u32) {
    let configs = vec![
        ProcessorCfg {
            xlen: XLen::X32,
            enable_dirty: true,
            extensions: vec!['m', 'a', 'c'].into_boxed_slice(),
            freq: 1000000000,
        };
        num_cores as usize
    ];

    let sys_bus = Rc::new(TerminusBus::new());
    let clint = Rc::new(Timer::new(100000000));
    let ext_bus = Box::new(ExtBus {
        name: "global".to_string(),
        id: 0x1000,
        base: 0x80000000,
        size: 0x80000000,
    });
    sys_bus
        .space_mut()
        .add_region(
            "ext_bus",
            &Region::remap(ext_bus.base, &Region::io(0, ext_bus.size, ext_bus)),
        )
        .unwrap();
    sys_bus
        .space_mut()
        .add_region(
            "clint",
            &Region::remap(0x02000000, &Region::io(0, 0x000c0000, Box::new(Clint::new(&clint)))),
        )
        .unwrap();
    for cfg in configs {
        let core_bus = Rc::new(CoreBus::new(
            &sys_bus,
            format!("core{}",unsafe { CLUSTER.processors.len() }),
            unsafe { CLUSTER.processors.len() } as u32,
            MemInfo {
                base: 0,
                size: 4096,
            },
            MemInfo {
                base: 4096,
                size: 4096 * 4,
            },
        ));
        let p = Processor::new(
            unsafe { CLUSTER.processors.len() },
            cfg,
            &core_bus,
            Some(clint.alloc_irq()),
            None,
        );
        unsafe { CLUSTER.processors.push(p) }
    }
    unsafe {
        CLUSTER.sys_bus = Some(sys_bus);
    };
}

#[no_mangle]
extern "C" fn cluster_reset_core(hartid: u32, boot_addr: u64) {
    unsafe {
        CLUSTER.processors[hartid as usize]
            .reset(boot_addr)
            .expect(format!("reset core {} to {:#x} fail!", hartid, boot_addr).as_str());
    }
    println!("reset core to {:#x}!", boot_addr);
}

#[no_mangle]
extern "C" fn cluster_run() -> ! {
    extern "C" {
        fn cluster_step();
    }
    unsafe {
        loop {
            for p in &mut CLUSTER.processors {
                p.step(1);
            }
            cluster_step();
        }
    }
}

#[no_mangle]
extern "C" fn cluster_run_1step() {
    unsafe {
        for p in &mut CLUSTER.processors {
            p.step(1);
        }
    }
}

#[no_mangle]
extern "C" fn cluster_statics() {
    unsafe {
        for p in &CLUSTER.processors {
            println!("{}", p.state().to_string())
        }
    }
}
