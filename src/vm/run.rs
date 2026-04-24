use kvm_ioctls::VcpuExit;

use crate::vm::vm::VirtualMachine;

pub enum CrashReason {
    Hlt,
    FailedEntry,
    UnhandledExit,
    NoIODataReturned,
    IncorrectIOInputLength,
    NoMMIODataReturned,
    IncorrectMMIOReadLength,
    Shutdown,
}

impl VirtualMachine {
    pub fn run(&mut self) -> Result<(), CrashReason> {
        let exit = self.vcpu.fd.run().expect("run failed");
        match exit {
            VcpuExit::Hlt => {
                println!("KVM_EXIT_HLT");
                return Err(CrashReason::Hlt);
            }
            VcpuExit::IoOut(port, data) => {
                let mut io_map = self.io_map.lock().unwrap();
                io_map.output(port, data);
            }
            VcpuExit::IoIn(port, data) => {
                let mut io_map = self.io_map.lock().unwrap();
                let io_ret = io_map.input(port, data.len());
                if io_ret.is_none() {
                    for b in data.iter_mut() {
                        *b = 0xFF;
                    }
                    return Ok(());
                }
                let io_ret = io_ret.unwrap();

                if io_ret.len() != data.len() {
                    println!("INCORRECT_IO_INPUT_LENGTH");
                    return Err(CrashReason::IncorrectIOInputLength);
                }
                data.copy_from_slice(&io_ret);
            }
            VcpuExit::MmioWrite(addr, data) => {
                let mut mmio_map = self.mmio_map.lock().unwrap();
                mmio_map.write(addr, data);
            }
            VcpuExit::MmioRead(addr, data) => {
                let mut mmio_map = self.mmio_map.lock().unwrap();
                let io_ret = mmio_map.read(addr, data.len());
                if io_ret.is_none() {
                    for b in data.iter_mut() {
                        *b = 0;
                    }
                    return Ok(());
                }
                let io_ret = io_ret.unwrap();

                if io_ret.len() != data.len() {
                    println!("INCORRECT_MMIO_INPUT_LENGTH");
                    return Err(CrashReason::IncorrectMMIOReadLength);
                }
                data.copy_from_slice(&io_ret);
            }
            VcpuExit::FailEntry(reason, ..) => {
                eprintln!("KVM_EXIT_FAIL_ENTRY: reason = {:#x}", reason);
                return Err(CrashReason::FailedEntry);
            }
            VcpuExit::Shutdown => {
                eprintln!("KVM_SHUTDOWN");
                let regs = self.vcpu.fd.get_regs().unwrap();
                let sregs = self.vcpu.fd.get_sregs().unwrap();
                eprintln!("SHUTDOWN at RIP={:#x}", regs.rip);
                eprintln!("RAX={:#x} RBX={:#x} RCX={:#x} RDX={:#x}", regs.rax, regs.rbx, regs.rcx, regs.rdx);
                eprintln!("CR0={:#x} CR3={:#x} CR4={:#x} EFER={:#x}", sregs.cr0, sregs.cr3, sregs.cr4, sregs.efer);
                eprintln!("CS base={:#x} selector={:#x} type={:#x} l={}", sregs.cs.base, sregs.cs.selector, sregs.cs.type_, sregs.cs.l);
                return Err(CrashReason::Shutdown);
            }
            exit_reason => {
                println!("Unhandled exit: {:?}", exit_reason);
                // return Err(CrashReason::UnhandledExit);
            }
        }
        Ok(())
    }
}