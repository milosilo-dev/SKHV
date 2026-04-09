use std::{cell::RefCell, ops::RangeInclusive, rc::Rc};

use crate::irq_handler::IRQHandler;

pub trait MMIODevice {
    fn read(&mut self, addr: u64, length: usize) -> Vec<u8>;
    fn write(&mut self, addr: u64, data: &[u8]);
    fn irq_handler(&mut self, irq_handler: Rc<RefCell<IRQHandler>>);
}

pub struct MMIODeviceRegion {
    mmio_device: Box<dyn MMIODevice>,
    addr_range: RangeInclusive<u64>
}

impl MMIODeviceRegion {
    pub fn new(range: RangeInclusive<u64>, device: Box<dyn MMIODevice>) -> Self {
        Self {
            mmio_device: device,
            addr_range: range
        }
    }

    pub fn contains(&self, addr: u64) -> bool {
        self.addr_range.contains(&addr)
    }

    pub fn read(&mut self, addr: u64, length: usize) -> Vec<u8> {
        self.mmio_device.read(addr - *self.addr_range.start(), length)
    }

    pub fn write(&mut self, addr: u64, data: &[u8]) {
        self.mmio_device.write(addr - *self.addr_range.start(), data);
    }

    pub fn irq_handler(&mut self, irq_handler: Rc<RefCell<IRQHandler>>) {
        self.mmio_device.irq_handler(irq_handler);
    }
}

pub struct MMIODeviceMap {
    devices: Vec<MMIODeviceRegion>,
}

impl MMIODeviceMap {
    pub fn new() -> Self {
        Self {
            devices: vec!{}
        }
    }

    pub fn register(&mut self, region: MMIODeviceRegion) {
        self.devices.push(region);
    }

    pub fn read(&mut self, addr: u64, length: usize) -> Option<Vec<u8>> {
        for device in &mut self.devices {
            if device.contains(addr){
                return Some(device.read(addr, length));
            }
        }
        None
    }

    pub fn write(&mut self, addr: u64, data: &[u8]) -> Option<()> {
        for device in &mut self.devices {
            if device.contains(addr){
                device.write(addr, data);
                return Some(());
            }
        }
        None
    }
}