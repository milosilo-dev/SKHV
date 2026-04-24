use std::{fs::{File, OpenOptions}, io::{Read, Seek, SeekFrom, Write}, path::Path};

use crate::devices::virtio::{virtio::{VirtioDevice, VirtioGuestMemoryHandle, VirtioQueue}};

const SECTOR_SIZE: u64 = 512;

pub struct BlkRequest{
    pub rqst_type: bool,
    pub sector: u64,
}

impl BlkRequest {
    pub fn new(base_ptr: u64, guest_memory: &VirtioGuestMemoryHandle) -> Self{
        let rqst_type = guest_memory.read_u32(base_ptr) != 0;
        let sector = guest_memory.read_u64(base_ptr + 8);
        Self { rqst_type, sector }
    }
}


pub struct BlkVirtio {
    guest_memory: Option<VirtioGuestMemoryHandle>,
    blk_file: File,
}

impl BlkVirtio {
    pub fn new(blk_file: &str) -> Self{
        let path = Path::new(blk_file);
        if !path.exists() {
            File::create(path).unwrap();
        }

        Self{
            guest_memory: None,
            blk_file: OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .expect("failed to open disk image"),
        }
    }
}

impl VirtioDevice for BlkVirtio {
    fn virtio_type(&self) -> u32 {
        0x04
    }

    fn features(&self) -> u32 {
        0
    }

    fn pass_guest_memory(&mut self, guest_memory: VirtioGuestMemoryHandle) {
        self.guest_memory = Some(guest_memory);
    }

    fn tick(&mut self, queue: &mut VirtioQueue) -> bool {
        if self.guest_memory.is_none(){
            return false;
        }

        let mut guest_memory = self.guest_memory.as_mut().unwrap();
        let mut did_work = false;

        while let Some(head) = queue.pop_avail(&guest_memory) {
            let header = queue.get_descriptor(&guest_memory, head);

            if header.flags & 1 == 0 { // Next
                panic!("virtio-blk got inncorect header");
            }

            if header.len != 16 {
                panic!("virtio-blk got the wrong header length");
            }

            let request = BlkRequest::new(header.addr, &guest_memory);

            let data_section = queue.get_descriptor(&guest_memory, header.next);

            if data_section.flags & 1 == 0 || data_section.flags & 2 == 0 { // Next & Write
                panic!("virtio-blk got inncorect data buffer");
            }

            let status_byte = queue.get_descriptor(&guest_memory, data_section.next);

            if data_section.flags & 2 == 0 {
                panic!("virtio-blk got inncorect status_byte");
            }

            match request.rqst_type {
                false => { // Device read
                    self.blk_file.seek(SeekFrom::Start(request.sector * SECTOR_SIZE)).unwrap();
                    let mut buf = vec![0u8; data_section.len as usize];

                    match self.blk_file.read_exact(&mut buf) {
                        Ok(_) => guest_memory.write_u8(status_byte.addr, 0x00),
                        Err(_) => guest_memory.write_u8(status_byte.addr, 0x01),
                    }

                    guest_memory.write_guest_memory(data_section.addr, &buf);
                },
                true => { // Device write
                    let mut buf = vec![0u8; data_section.len as usize];
                    guest_memory.read_guest_memory(data_section.addr, &mut buf);
                    self.blk_file.seek(SeekFrom::Start(request.sector * 512)).unwrap(); 

                    match self.blk_file.write_all(&buf) {
                        Ok(_) => guest_memory.write_u8(status_byte.addr, 0x00),
                        Err(_) => guest_memory.write_u8(status_byte.addr, 0x01),
                    }
                }
            }

            queue.push_used(&mut guest_memory, head, data_section.len);

            did_work = true;
        }

        did_work
    }
}