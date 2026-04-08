use std::fs;

use skhv::{device_maps::io::IODeviceRegion, devices::serial::Serial, vm::VirtualMachine};

fn main() {
    let init_mem_image = fs::read("guest/firmware.bin").unwrap();
    let mut vm = VirtualMachine::new(Vec::from(init_mem_image));

    let com1 = Box::new(Serial::new());
    vm.register_io_device(IODeviceRegion::new(0x3f8..=0x3ff, com1));

    let com2 = Box::new(Serial::new());
    vm.register_io_device(IODeviceRegion::new(0x2f8..=0x2ff, com2));

    loop {
        let ret = vm.run();
        if ret.is_err() {
            break;
        }
    }
}