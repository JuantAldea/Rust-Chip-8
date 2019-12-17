use rand::Rng;
use std::time::Instant;
extern crate log;
use log::{debug, info};

static FONT: [u8; 5 * 16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    pub memory: [u8; 4096],
    pub video_memory: [bool; 64 * 32],
    pub stack: [usize; 16],
    pub v: [u8; 16],
    pub keys: [bool; 16],
    pub i: usize,
    pub dt: u8,
    pub st: u8,
    pub pc: usize,
    pub sp: usize,
    pub timer_counter: usize,
    pub font_base_addr: usize,
    pub waiting_for_key: bool,
    pub read_key_registry: usize,
    pub interrupted: bool,
    pub rendered: bool,
}

#[derive(Debug)]
pub enum Chip8Exception {
    StackOverflow,
    StackUnderflow,
    MemoryOverflow,
    MemoryUnderflow,
    InvalidInstruction,
    UHM,
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8::default()
    }

    pub fn init(&mut self) {
        self.memory[self.font_base_addr..self.font_base_addr + 80].copy_from_slice(&FONT);
        for i in self.video_memory.iter_mut() {
            *i = false
        }
    }

    pub fn load_rom(&mut self, addr: usize, program: &[u8]) {
        self.memory[addr..].copy_from_slice(program);
    }

    pub fn int(&mut self) {
        self.interrupted = !self.interrupted;
    }

    pub fn tick(&mut self, input: &[bool]) {
        self.read_input(input);
        debug!("{}", self);
        self.cycle();
    }

    pub fn read_input(&mut self, input: &[bool]) {
        for (i, item) in input.iter().enumerate().take(16) {
            self.keys[i] = *item;
            if *item && self.waiting_for_key {
                self.v[self.read_key_registry] = i as u8;
                self.waiting_for_key = false;
            }
        }
    }

    pub fn cycle(&mut self) {
        if self.interrupted {
            return;
        }

        if self.waiting_for_key {
            info!("Waiting for keypress");
            return;
        }

        let _now = Instant::now();
        self.rendered = false;
        let op_code = self.fetch_instruction().unwrap();
        self.decode_and_exec_instruction(op_code).unwrap();
        self.update_timers();
        //println!("Cycle: {}us", _now.elapsed().as_micros());
    }

    pub fn update_timers(&mut self) {
        if self.timer_counter == 10 {
            if self.dt > 0 {
                self.dt -= 1;
            }

            if self.st > 0 {
                self.dt -= 1;
            }
            self.timer_counter = 0;
        } else {
            self.timer_counter += 1;
        }
    }

    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }

    pub fn decode_and_exec_instruction(&mut self, op_code: u16) -> Result<(), Chip8Exception> {
        let nibbles = [
            ((op_code & 0xf000) >> 12) as usize,
            ((op_code & 0x0f00) >> 8) as usize,
            ((op_code & 0x00f0) >> 4) as usize,
            (op_code & 0x000f) as usize,
        ];

        let nnn = (op_code & 0x0FFF) as usize;
        let kk = (op_code & 0x00FF) as u8;

        match nibbles {
            [0x0, 0x0, 0xE, 0x0] => self.cls(),
            [0x0, 0x0, 0xE, 0xE] => self.ret(),
            [0x0, _, _, _] => self.sys_addr(nnn),
            [0x1, _, _, _] => self.jp_addr(nnn),
            [0x2, _, _, _] => self.call_addr(nnn),
            [0x3, vx, _, _] => self.se_vx_byte(vx, kk),
            [0x4, vx, _, _] => self.sne_vx_byte(vx, kk),
            [0x5, vx, vy, 0x0] => self.se_vx_vy(vx, vy),
            [0x6, vx, _, _] => self.ld_vx_byte(vx, kk),
            [0x7, vx, _, _] => self.add_vx_byte(vx, kk),
            [0x8, vx, vy, 0x0] => self.ld_vx_vy(vx, vy),
            [0x8, vx, vy, 0x1] => self.or_vx_vy(vx, vy),
            [0x8, vx, vy, 0x2] => self.and_vx_vy(vx, vy),
            [0x8, vx, vy, 0x3] => self.xor_vx_vy(vx, vy),
            [0x8, vx, vy, 0x4] => self.add_vx_vy(vx, vy),
            [0x8, vx, vy, 0x5] => self.sub_vx_vy(vx, vy),
            [0x8, vx, vy, 0x6] => self.shr_vx_vy(vx, vy),
            [0x8, vx, vy, 0x7] => self.subn_vx_vy(vx, vy),
            [0x8, vx, vy, 0xE] => self.shl_vx_vy(vx, vy),
            [0x9, vx, vy, 0x0] => self.sne_vx_vy(vx, vy),
            [0xA, _, _, _] => self.ld_i_addr(nnn),
            [0xB, _, _, _] => self.jp_v0_addr(nnn),
            [0xC, vx, _, _] => self.rnd_vx_byte(vx, kk),
            [0xD, vx, vy, n] => self.drw_vx_vy_nibble(vx, vy, n as u8),
            [0xE, vx, 0x9, 0xE] => self.skp_vx(vx),
            [0xE, vx, 0xA, 0x1] => self.sknp_vx(vx),
            [0xF, vx, 0x0, 0x7] => self.ld_vx_dt(vx),
            [0xF, vx, 0x0, 0xA] => self.ld_vx_k(vx),
            [0xF, vx, 0x1, 0x5] => self.ld_dt_vx(vx),
            [0xF, vx, 0x1, 0x8] => self.ld_st_vx(vx),
            [0xF, vx, 0x1, 0xE] => self.add_i_vx(vx),
            [0xF, vx, 0x2, 0x9] => self.ld_f_vx(vx),
            [0xF, vx, 0x3, 0x3] => self.ld_b_vx(vx),
            [0xF, vx, 0x5, 0x5] => self.ld_mem_i_vx(vx),
            [0xF, vx, 0x6, 0x5] => self.ld_vx_mem_i(vx),
            _ => Err(Chip8Exception::InvalidInstruction),
        }
    }

    pub fn stack_push(&mut self, value: usize) -> Result<(), Chip8Exception> {
        if self.sp >= 16 {
            return Err(Chip8Exception::StackOverflow);
        }
        self.stack[self.sp] = value;
        self.sp += 1;
        Ok(())
    }

    pub fn stack_pop(&mut self) -> Result<usize, Chip8Exception> {
        match self.sp.checked_sub(1) {
            Some(value) => {
                self.sp = value;
                let ret_addr = self.stack[self.sp];
                Ok(ret_addr)
            }
            None => Err(Chip8Exception::StackUnderflow),
        }
    }

    pub fn fetch_instruction(&mut self) -> Result<u16, Chip8Exception> {
        if self.pc + 2 >= 4096 {
            return Err(Chip8Exception::MemoryOverflow);
        }

        let op_code = (self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16;
        debug!(
            "=PC:0x{:04x} -> 0x{:04x} ({:x}|{:x}) => ",
            self.pc,
            op_code,
            self.memory[self.pc],
            self.memory[self.pc + 1]
        );
        self.pc += 2;
        Ok(op_code)
    }

    pub fn cls(&mut self) -> Result<(), Chip8Exception> {
        debug!("CLS");
        for i in self.video_memory.iter_mut() {
            *i = false
        }
        self.rendered = true;
        Ok(())
    }

    pub fn ret(&mut self) -> Result<(), Chip8Exception> {
        debug!("RET");
        match self.stack_pop() {
            Ok(value) => {
                self.pc = value;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn sys_addr(&mut self, addr: usize) -> Result<(), Chip8Exception> {
        debug!("SYS addr({:x}) => NOP", addr);
        Ok(())
    }

    pub fn jp_addr(&mut self, addr: usize) -> Result<(), Chip8Exception> {
        debug!("JP addr({:x})", addr);
        self.pc = addr;
        Ok(())
    }

    pub fn call_addr(&mut self, addr: usize) -> Result<(), Chip8Exception> {
        debug!("CALL addr({:x})", addr);
        self.stack_push(self.pc).unwrap();
        self.pc = addr;
        Ok(())
    }

    pub fn se_vx_byte(&mut self, vx: usize, byte: u8) -> Result<(), Chip8Exception> {
        debug!("SE V{:x}({:x}), byte({:x})", vx, self.v[vx], byte);
        if self.v[vx] == byte {
            self.pc += 2
        }
        Ok(())
    }

    pub fn sne_vx_byte(&mut self, vx: usize, byte: u8) -> Result<(), Chip8Exception> {
        debug!("SNE V{:x}, byte({:x})", vx, byte);
        if self.v[vx] != byte {
            self.pc += 2
        }
        Ok(())
    }

    pub fn se_vx_vy(&mut self, vx: usize, vy: usize) -> Result<(), Chip8Exception> {
        debug!("SE V{:x}, V{:x}", vx, vy);
        if self.v[vx] == self.v[vy] {
            self.pc += 2
        }
        Ok(())
    }

    pub fn ld_vx_byte(&mut self, vx: usize, byte: u8) -> Result<(), Chip8Exception> {
        debug!("LD V{:x}, byte({:x})", vx, byte);
        self.v[vx] = byte;
        Ok(())
    }

    pub fn add_vx_byte(&mut self, vx: usize, byte: u8) -> Result<(), Chip8Exception> {
        debug!("ADD V{:x}, byte({:x})", vx, byte);
        let (value, _) = self.v[vx].overflowing_add(byte);
        self.v[vx] = value;
        Ok(())
    }

    pub fn ld_vx_vy(&mut self, vx: usize, vy: usize) -> Result<(), Chip8Exception> {
        debug!("LD V{:x}, V{:x}", vx, vy);
        self.v[vx] = self.v[vy];
        Ok(())
    }

    pub fn or_vx_vy(&mut self, vx: usize, vy: usize) -> Result<(), Chip8Exception> {
        debug!("OR V{:x}, V{:x}", vx, vy);
        self.v[vx] |= self.v[vy];
        Ok(())
    }

    pub fn and_vx_vy(&mut self, vx: usize, vy: usize) -> Result<(), Chip8Exception> {
        debug!("AND V{:x}, V{:x}", vx, vy);
        self.v[vx] &= self.v[vy];
        Ok(())
    }

    pub fn xor_vx_vy(&mut self, vx: usize, vy: usize) -> Result<(), Chip8Exception> {
        debug!("XOR V{:x}, V{:x}", vx, vy);
        self.v[vx] ^= self.v[vy];
        Ok(())
    }

    pub fn add_vx_vy(&mut self, vx: usize, vy: usize) -> Result<(), Chip8Exception> {
        debug!("ADD V{:x}, V{:x}", vx, vy);
        let (value, overflow) = self.v[vx].overflowing_add(self.v[vy]);
        self.v[0xF] = if overflow { 1 } else { 0 };
        self.v[vx] = value;
        Ok(())
    }

    pub fn sub_vx_vy(&mut self, vx: usize, vy: usize) -> Result<(), Chip8Exception> {
        debug!("SUB V{:x}, V{:x}", vx, vy);
        self.v[0xF] = if self.v[vx] > self.v[vy] { 1 } else { 0 };
        self.v[vx] = self.v[vx].wrapping_sub(self.v[vy]);
        Ok(())
    }

    pub fn shr_vx_vy(&mut self, vx: usize, vy: usize) -> Result<(), Chip8Exception> {
        debug!("SHR V{:x} {{, V{:x}}}", vx, vy);
        let (value, overflow) = self.v[vx].overflowing_shr(1);
        self.v[0xF] = if overflow { 1 } else { 0 };
        self.v[vx] = value;
        Ok(())
    }

    pub fn subn_vx_vy(&mut self, vx: usize, vy: usize) -> Result<(), Chip8Exception> {
        debug!("SUBN V{:x}, V{:x}", vx, vy);
        self.v[0xF] = if self.v[vy] > self.v[vx] { 1 } else { 0 };
        self.v[vx] = self.v[vy].wrapping_sub(self.v[vx]);
        Ok(())
    }

    pub fn shl_vx_vy(&mut self, vx: usize, vy: usize) -> Result<(), Chip8Exception> {
        debug!("SHL V{:x} '{{, V{:x}}}", vx, vy);
        let (value, overflow) = self.v[vx].overflowing_shl(1);
        self.v[0xF] = if overflow { 1 } else { 0 };
        self.v[vx] = value;
        Ok(())
    }

    pub fn sne_vx_vy(&mut self, vx: usize, vy: usize) -> Result<(), Chip8Exception> {
        debug!("SNE V{:x}, V{:x}", vx, vy);
        if self.v[vx] != self.v[vy] {
            self.pc += 2
        }
        Ok(())
    }

    pub fn ld_i_addr(&mut self, addr: usize) -> Result<(), Chip8Exception> {
        debug!("LD I, addr({:x})", addr);
        self.i = addr;
        Ok(())
    }

    pub fn jp_v0_addr(&mut self, addr: usize) -> Result<(), Chip8Exception> {
        debug!("JP V0, addr({:x})", addr);
        self.pc = addr + self.v[0 as usize] as usize;
        Ok(())
    }

    pub fn rnd_vx_byte(&mut self, vx: usize, byte: u8) -> Result<(), Chip8Exception> {
        let n: u8 = rand::thread_rng().gen();
        self.v[vx] = n & byte;
        debug!("RND V{:x}, byte({:x}) => {:x}", vx, byte, self.v[vx]);
        Ok(())
    }

    pub fn drw_vx_vy_nibble(
        &mut self,
        vx: usize,
        vy: usize,
        nibble: u8,
    ) -> Result<(), Chip8Exception> {
        debug!("DRW V{:x}, V{:x}, nibble({:x})", vx, vy, nibble);
        self.v[0xF] = 0;
        for y in 0..nibble {
            let pixel_row = self.memory[self.i + y as usize];
            let vm_row = (self.v[vy] as usize + y as usize) % 32;
            for x in 0..8 {
                let vm_col = (self.v[vx] as usize + x as usize) % 64;
                let pixel_value = (pixel_row & (0x80 >> x)) != 0;
                if pixel_value {
                    let position = 64 * vm_row + vm_col;
                    if self.video_memory[position] {
                        self.v[0xF] = 1;
                    }
                    self.video_memory[position] ^= pixel_value;
                }
            }
        }
        self.rendered = true;
        Ok(())
    }

    pub fn skp_vx(&mut self, vx: usize) -> Result<(), Chip8Exception> {
        debug!("SKP V{:x}", vx);

        if self.keys[self.v[vx] as usize] {
            self.pc += 2;
        }

        Ok(())
    }

    pub fn sknp_vx(&mut self, vx: usize) -> Result<(), Chip8Exception> {
        debug!("SKNP V{:x}", vx);
        if !self.keys[self.v[vx] as usize] {
            self.pc += 2;
        }
        Ok(())
    }

    pub fn ld_vx_dt(&mut self, vx: usize) -> Result<(), Chip8Exception> {
        debug!("LD V{:x}, DT", vx);
        self.v[vx] = self.dt;
        Ok(())
    }

    pub fn ld_vx_k(&mut self, vx: usize) -> Result<(), Chip8Exception> {
        debug!("LD V{:x}, K", vx);
        self.waiting_for_key = true;
        self.read_key_registry = vx;
        Ok(())
    }

    pub fn ld_dt_vx(&mut self, vx: usize) -> Result<(), Chip8Exception> {
        debug!("LD DT, V{:x}", vx);
        self.dt = self.v[vx];
        Ok(())
    }

    pub fn ld_st_vx(&mut self, vx: usize) -> Result<(), Chip8Exception> {
        debug!("LD ST, V{:x}", vx);
        self.st = self.v[vx];
        Ok(())
    }

    pub fn add_i_vx(&mut self, vx: usize) -> Result<(), Chip8Exception> {
        debug!("ADD I, V{:x}", vx);
        self.i += self.v[vx] as usize;
        Ok(())
    }

    pub fn ld_f_vx(&mut self, vx: usize) -> Result<(), Chip8Exception> {
        debug!("LD F, V{:x}", vx);
        self.i = self.font_base_addr + 5 * self.v[vx] as usize;
        Ok(())
    }

    pub fn ld_b_vx(&mut self, vx: usize) -> Result<(), Chip8Exception> {
        debug!("LD B, V{:x}", vx);
        self.memory[self.i] = (vx / 100) as u8;
        self.memory[self.i + 1] = (vx % 100) as u8;
        self.memory[self.i + 2] = (vx % 10) as u8;
        Ok(())
    }

    pub fn ld_mem_i_vx(&mut self, vx: usize) -> Result<(), Chip8Exception> {
        debug!("LD [I], V{:x}", vx);
        for r in 0..=vx {
            self.memory[self.i as usize + r] = self.v[r];
        }
        Ok(())
    }

    pub fn ld_vx_mem_i(&mut self, vx: usize) -> Result<(), Chip8Exception> {
        debug!("LD V{:x}, [I]", vx);
        for r in 0..=vx {
            self.v[r] = self.memory[self.i as usize + r]
        }
        Ok(())
    }

    /*
    // SuperChip-8
    pub fn scd(&mut self) { debug!("SCD nibble"); }
    pub fn scr(&mut self) { debug!("SCR"); }
    pub fn scl(&mut self) { debug!("SCL"); }
    pub fn exit(&mut self) { debug!("EXIT"); }
    pub fn low(&mut self) { debug!("LOW"); }
    pub fn high(&mut self) { debug!("HIGH"); }
    pub fn drw_vx_vy_0(&mut self) { debug!("DRW Vx, Vy, 0"); }
    pub fn ld_hf_vx(&mut self) { debug!("LD HF, Vx"); }
    pub fn ld_r_vx(&mut self) { debug!("LD R, Vx"); }
    pub fn ld_vx_r(&mut self) { debug!("LD Vx, R"); }
    */
}

use std::fmt;

impl fmt::Display for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "=-------------------------------------------------").unwrap();
        writeln!(f, "=Mx{:x?}", &self.memory[self.pc..self.pc + 6]).unwrap();
        writeln!(f, "=Px{:?}", &self.video_memory[..10]).unwrap();
        writeln!(f, "=Sx{:x?}", &self.stack).unwrap();
        writeln!(f, "=Vx{:x?} ", &self.v).unwrap();
        writeln!(f, "=Kx{:x?} ", &self.keys).unwrap();
        writeln!(
            f,
            "=PC: {:x}, SP: {:x}, DT: {:x}, ST: {:x} I: {:x} TC: {:}",
            &self.pc, &self.sp, &self.dt, &self.st, &self.i, &self.timer_counter
        )
        .unwrap();
        write!(f, "-------------------------------------------------")
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Self {
            memory: [0; 4096],
            v: [0; 16],
            stack: [0; 16],
            video_memory: [false; 64 * 32],
            i: 0,
            dt: 0,
            st: 0,
            pc: 0,
            sp: 0,
            keys: [false; 16],
            timer_counter: 10,
            font_base_addr: 0,
            waiting_for_key: false,
            read_key_registry: 0,
            interrupted: false,
            rendered: false,
        }
    }
}
