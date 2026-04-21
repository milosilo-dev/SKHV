use crate::{
    device_maps::{io::IODeviceRegion, mmio::MMIODeviceRegion},
    irq_map::IrqMap,
};

pub struct MemoryRegionConfig {
    pub mem_size: usize,
    pub mem_offset: u64,
}

pub struct Binary {
    pub data: Vec<u8>,
    pub offset: u64,
}

impl Binary {
    pub fn new(data: Vec<u8>, offset: u64) -> Self {
        Self { data, offset }
    }

    fn build_boot_params(bzimage: &[u8]) -> Vec<u8> {
        // struct boot_params / "zero page" — see Documentation/x86/boot.rst
        let mut bp = vec![0u8; 4096]; // 4 KB, fully zeroed

        // ── Copy the setup_header verbatim from the bzImage ────────────────
        // The header starts at offset 0x1F1 and extends through 0x26F
        // (kernel_info_offset at 0x268, protocol 2.15).  0x7F bytes covers
        // everything safely without overrunning into reserved territory.
        let hdr_src = 0x1F1usize;
        let hdr_len = 0x7F;
        bp[hdr_src..hdr_src + hdr_len]
            .copy_from_slice(&bzimage[hdr_src..hdr_src + hdr_len]);

        // ── Fields the boot loader MUST write (boot protocol §1.3) ─────────

        // type_of_loader (0x210): 0xFF = unknown/custom boot loader.
        // If left at 0x00 the kernel may reject the boot on newer versions.
        bp[0x210] = 0xFF;

        // loadflags (0x211):
        //   bit 0 (LOADED_HIGH)  – protected-mode kernel is at 0x100000 ✓
        //   bit 7 (CAN_USE_HEAP) – we supply a valid heap_end_ptr below
        let loadflags = bp[0x211];
        bp[0x211] = loadflags | 0x01 | 0x80;

        // heap_end_ptr (0x224): highest usable address *within* the real-mode
        // segment (relative to the start of the segment, i.e. offset from X).
        // Setup is at X=0x10000; 0x7FF0 gives the kernel ~28 KB of heap
        // between the end of setup code and the stack guard.
        let heap_end: u16 = 0x7FF0;
        bp[0x224..0x226].copy_from_slice(&heap_end.to_le_bytes());

        // vid_mode (0x1FA): 0xFFFF = "normal" / don't change the video mode.
        // The boot loader is required to set this field explicitly.
        let vid: u16 = 0xFFFF;
        bp[0x1FA..0x1FC].copy_from_slice(&vid.to_le_bytes());

        // cmd_line_ptr (0x228): 32-bit linear address of the command line.
        let cmdline_addr: u32 = 0x21000;
        bp[0x228..0x22C].copy_from_slice(&cmdline_addr.to_le_bytes());

        // cmdline_size (0x238): actual byte-length of our command line string,
        // NOT the kernel's reported maximum.  Exclude the NUL terminator.
        const CMDLINE: &[u8] = b"console=ttyS0 earlyprintk=serial\0";
        let size = (CMDLINE.len() - 1) as u32;
        bp[0x238..0x23C].copy_from_slice(&size.to_le_bytes());

        bp
    }

    pub fn load_bz_image(data: Vec<u8>) -> Vec<Self> {
        let setup_sects = if data[0x1F1] == 0 {
            4
        } else {
            data[0x1F1] as usize
        };

        let setup_size = (setup_sects + 1) * 512;
        let kernel_offset = setup_size;

        let boot_params = Self::build_boot_params(&data);
        vec![
            Self {
                data: data[0..setup_size].to_vec(),
                offset: 0x10000,
            },
            Self {
                data: data[kernel_offset..].to_vec(),
                offset: 0x100000,
            },
            Self {
                data: boot_params,
                offset: 0x20000,
            },
            Self {
                data: b"console=ttyS0 earlyprintk=serial\0".to_vec(),
                offset: 0x21000,
            },
        ]
    }
}

pub struct MachineConfig {
    pub memory_regions: Vec<MemoryRegionConfig>,
    pub binaries: Vec<Binary>,
    pub io_devices: Vec<IODeviceRegion>,
    pub mmio_devices: Vec<MMIODeviceRegion>,
    pub irq_map: Vec<IrqMap>,

    pub code_entry: usize,
}