use kvm_bindings::{
    kvm_userspace_memory_region
};

use kvm_ioctls::{
    Kvm, 
    VcpuExit, VmFd,
};

use crate::{device_maps::{
    io::{
        IODeviceMap, 
        IODeviceRegion
    }, 
    mmio::{
        MMIODeviceMap, 
        MMIODeviceRegion
    }
}, irq_handler::IRQHandler, machine_config::MachineConfig, vcpu::VCPU};
use std::{cell::RefCell, ptr, rc::Rc};
use libc::{MAP_ANONYMOUS, MAP_PRIVATE, PROT_READ, PROT_WRITE, mmap};

pub enum CrashReason {
    Hlt,
    FailedEntry,
    UnhandledExit,
    NoIODataReturned,
    IncorrectIOInputLength,
    NoMMIODataReturned,
    IncorrectMMIOReadLength,
}

pub struct VirtualMachine{
    vcpu: VCPU,
    vm: VmFd,
    io_map: IODeviceMap,
    mmio_map: MMIODeviceMap,
    memory_regions: Vec<*mut u8>,
    irq_handler: Rc<RefCell<IRQHandler>>
}

impl VirtualMachine{
    pub fn new(init_mem_image: Vec<u8>, machine_config: MachineConfig) -> Self{
        let kvm: Kvm = Kvm::new().unwrap();
        let vm = kvm.create_vm().unwrap();
        vm.create_irq_chip().unwrap();

        let io_map = IODeviceMap::new();
        let mmio_map = MMIODeviceMap::new();
        let irq_handler = Rc::new(RefCell::new(IRQHandler::new()));

        let vcpu = VCPU::new(&vm, machine_config.code_entry);
        let mut this = Self {
            vcpu,
            vm,
            io_map,
            mmio_map,
            memory_regions: vec![],
            irq_handler: Rc::clone(&irq_handler),
        };

        for mem in machine_config.memory_regions {
            this.new_mem(mem.mem_size, mem.mem_offset);
            if mem.mem_offset <= machine_config.code_entry as u64
                && mem.mem_offset + mem.mem_size as u64 > machine_config.code_entry as u64
            {
                let code_offset = machine_config.code_entry as usize - mem.mem_offset as usize;
                let remaining = mem.mem_size.checked_sub(code_offset)
                    .expect("code_entry offset exceeds memory region size");

                assert!(
                    init_mem_image.len() <= remaining,
                    "init_mem_image ({} bytes) overflows memory region: only {} bytes available from code entry (offset {:#x}) to end of region",
                    init_mem_image.len(),
                    remaining,
                    code_offset,
                );

                unsafe {
                    ptr::copy_nonoverlapping(
                        init_mem_image.as_ptr(),
                        this.memory_regions.last().expect("Can't find memory region").add(code_offset),
                        init_mem_image.len()
                    );
                }
            }
        }

        for mut mmio_device in machine_config.mmio_devices {
            mmio_device.irq_handler(Rc::clone(&irq_handler));
            this.register_mmio_device(mmio_device);
        }

        for mut io_device in machine_config.io_devices {
            io_device.irq_handler(Rc::clone(&irq_handler));
            this.register_io_device(io_device);
        }

        this
    }

    fn new_mem(&mut self, mem_size: usize, mem_offset: u64) {
        let raw_ptr = unsafe {
            mmap(
                std::ptr::null_mut(),
                mem_size,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        if raw_ptr == libc::MAP_FAILED {
            panic!("mmap failed");
        }

        let userspace_mem = raw_ptr as *mut u8;
        self.memory_regions.push(userspace_mem);

        let memory_region = kvm_userspace_memory_region{
            slot: self.memory_regions.len() as u32 - 1,
            flags: 0,
            guest_phys_addr: mem_offset,
            memory_size: mem_size as u64,
            userspace_addr: userspace_mem as u64
        };

        let _mem = unsafe { self.vm.set_user_memory_region(memory_region) }.unwrap();
    }

    fn register_io_device(&mut self, region: IODeviceRegion) {
        self.io_map.register(region);
    }

    fn register_mmio_device(&mut self, region: MMIODeviceRegion) {
        self.mmio_map.register(region);
    }

    pub fn run(&mut self) -> Result<(), CrashReason> {
        let mut irqs = self.irq_handler.borrow_mut().handle_irqs();
        for _ in 0..irqs.len(){
            let irq = irqs.pop_front();
            if irq.is_some(){
                let irq = irq.unwrap();
                let res = self.vm.set_irq_line(irq.irq_line, irq.value);
                if res.is_err() {
                    eprintln!("IRQ failed!");
                }
            }
            println!("irq");
        }

        println!("run");
        let ret = self.vcpu.run();
        println!("run complete");
        match ret {
            VcpuExit::Hlt => {
                println!("KVM_EXIT_HLT");
                return Err(CrashReason::Hlt);
            }
            VcpuExit::IoOut(port, data) => {
                if port == 0xFFFF {
                    println!("KVM_EXIT_HLT");
                    return Err(CrashReason::Hlt);
                }
                self.io_map.output(port, data);
            }
            VcpuExit::IoIn(port, data) => {
                let ret = self.io_map.input(port, data.len());
                if ret.is_none() {
                    println!("NO_IO_DATA_RETURNED");
                    return Err(CrashReason::NoIODataReturned);
                }
                let ret = ret.unwrap();

                if ret.len() != data.len() {
                    println!("INCORRECT_IO_INPUT_LENGTH");
                    return Err(CrashReason::IncorrectIOInputLength);
                }
                data.copy_from_slice(&ret);
            }
            VcpuExit::MmioWrite(addr, data) => {
                self.mmio_map.write(addr, data);
            }
            VcpuExit::MmioRead(addr, data) => {
                let ret = self.mmio_map.read(addr, data.len());
                if ret.is_none() {
                    println!("NO_MMIO_DATA_RETURNED");
                    return Err(CrashReason::NoMMIODataReturned);
                }
                let ret = ret.unwrap();

                if ret.len() != data.len() {
                    println!("INCORRECT_MMIO_INPUT_LENGTH");
                    return Err(CrashReason::IncorrectMMIOReadLength);
                }
                data.copy_from_slice(&ret);
            }
            VcpuExit::FailEntry(reason, ..) => {
                eprintln!(
                    "KVM_EXIT_FAIL_ENTRY: reason = {:#x}",
                    reason
                );
                return Err(CrashReason::FailedEntry);
            }
            exit_reason => {
                println!("Unhandled exit: {:?}", exit_reason);
                return Err(CrashReason::UnhandledExit);
            }
        }
        Ok(())
    }
}