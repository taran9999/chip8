use rand::{rngs::ThreadRng, Rng};
use sdl2::keyboard::Keycode;

pub struct Chip8 {
    memory: [u8; 4096], // 4096 bytes
    display: [u8; 64 * 32], // 64 * 32 pixels
    pc: u16, // program counter
    i: u16, // memory pointer
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    v: [u8; 16], // variable registers
    modern: bool, // toggle for modern implementations of ambiguous instructions
    rng: ThreadRng,
    keypad: [bool; 16],
    key_pressed: bool,
    pressed_key: usize
}

pub enum ExecutionEffect {
    NoEffect,
    DisplayUpdate,
    JumpToSelf,
    WaitingForKey,
    Sound
}

const FONT: [u8; 80] = [
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

impl Chip8 {
    pub fn new(modern: bool) -> Chip8 {
        Chip8 {
            memory: [0; 4096],
            display: [0; 64 * 32],
            pc: 0x200, // instructions start at 0x200
            i: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            v: [0; 16],
            modern: modern,
            rng: rand::thread_rng(),
            keypad: [false; 16],
            key_pressed: false,
            pressed_key: 0
        }
    }

    pub fn init(&mut self) {
        // Load font into memory, from beginning
        for (i, &b) in FONT.iter().enumerate() {
            self.memory[i] = b;
        }
    }

    // Load Chip8 binary into memory starting at 0x200
    pub fn load_bin(&mut self, bin: &[u8]) {
        for (i, &b) in bin.iter().enumerate() {
            self.memory[0x200 + i] = b;
        }
    }

    pub fn display(&self) -> &[u8; 64 * 32] {
        &self.display
    }

    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 { self.delay_timer -= 1; }
        if self.sound_timer > 0 { self.sound_timer -= 1; }
    }

    pub fn sound_timer(&self) -> u8 {
        self.sound_timer
    }

    pub fn key_down(&mut self, key: Keycode) {
        println!("Key down: {key}");
        match key {
            Keycode::Num1 => self.press_key(0x1), // 1
            Keycode::Num2 => self.press_key(0x2), // 2
            Keycode::Num3 => self.press_key(0x3), // 3
            Keycode::Num4 => self.press_key(0xC), // C
            Keycode::Q => self.press_key(0x4), // 4
            Keycode::W => self.press_key(0x5), // 5
            Keycode::E => self.press_key(0x6), // 6
            Keycode::R => self.press_key(0xD), // D
            Keycode::A => self.press_key(0x7), // 7
            Keycode::S => self.press_key(0x8), // 8
            Keycode::D => self.press_key(0x9), // 9
            Keycode::F => self.press_key(0xE), // E
            Keycode::Z => self.press_key(0xA), // A
            Keycode::X => self.press_key(0x0), // 0
            Keycode::C => self.press_key(0xB), // B
            Keycode::V => self.press_key(0xF), // F
            _ => println!("Unmapped key: {key}")
        }
    }

    fn press_key(&mut self, n: usize) {
        self.keypad[n] = true;
        self.pressed_key = n;
        self.key_pressed = true;
    }

    pub fn key_up(&mut self, key: Keycode) {
        println!("Key up: {key}");
        match key {
            Keycode::Num1 => self.keypad[0x1] = false, // 1
            Keycode::Num2 => self.keypad[0x2] = false, // 2
            Keycode::Num3 => self.keypad[0x3] = false, // 3
            Keycode::Num4 => self.keypad[0xC] = false, // C
            Keycode::Q => self.keypad[0x4] = false, // 4
            Keycode::W => self.keypad[0x5] = false, // 5
            Keycode::E => self.keypad[0x6] = false, // 6
            Keycode::R => self.keypad[0xD] = false, // D
            Keycode::A => self.keypad[0x7] = false, // 7
            Keycode::S => self.keypad[0x8] = false, // 8
            Keycode::D => self.keypad[0x9] = false, // 9
            Keycode::F => self.keypad[0xE] = false, // E
            Keycode::Z => self.keypad[0xA] = false, // A
            Keycode::X => self.keypad[0x0] = false, // 0
            Keycode::C => self.keypad[0xB] = false, // B
            Keycode::V => self.keypad[0xF] = false, // F
            _ => println!("Unmapped key: {key}")
        }
    }

    // Fetch the instruction from memory at program counter
    pub fn fetch(&mut self) -> u16 {
        // Get both bytes, combine to a single 16 bit value and return
        let b1 = self.memory[self.pc as usize] as u16;
        let b2 = self.memory[(self.pc + 1) as usize] as u16;
        
        self.pc += 2;

        // Left shift first byte by 8, OR with second byte to get both in one
        (b1 << 8) | b2
    }

    pub fn execute(&mut self, opcode: u16) -> ExecutionEffect {
        // Extract 16 bits into 4 nibbles
        let n1 = opcode & 0xF000;
        let n2 = opcode & 0x0F00;
        let n3 = opcode & 0x00F0;
        let n4 = opcode & 0x000F;

        match opcode {
            // 00E0: Clear screen
            0x00E0 => {
                println!("Opcode: {:#X} (Clear screen)", opcode);
                self.display.fill(0);
            }

            // 00EE: Return
            0x00EE => {
                println!("Opcode: {:#X} (Return)", opcode);
                self.pc = self.stack.pop().expect("Stack empty.");
            }

            _ => {
                match n1 {
                    // 1NNN: jump to 0x0NNN
                    0x1000 => {
                        println!("Opcode: {:#X} (Jump)", opcode);
                        let old_pc = self.pc;
                        self.pc = n2 | n3 | n4;
                        if self.pc == old_pc - 2 {
                            return ExecutionEffect::JumpToSelf;
                        }
                    }

                    // 2NNN: push to stack and jump
                    0x2000 => {
                        println!("Opcode: {:#X} (Subroutine)", opcode);
                        self.stack.push(self.pc);
                        self.pc = n2 | n3 | n4;
                    }

                    // 3XNN: skip next if v[X] = NN
                    0x3000 => {
                        println!("Opcode: {:#X} (Skip Equal)", opcode);
                        let r = (n2 >> 8) as usize;
                        let val = (n3 | n4) as u8;
                        if self.v[r] == val {
                            self.pc += 2;
                        }
                    }

                    // 4XNN: skip next if v[X] != NN
                    0x4000 => {
                        println!("Opcode: {:#X} (Skip Not Equal)", opcode);
                        let r = (n2 >> 8) as usize;
                        let val = (n3 | n4) as u8;
                        if self.v[r] != val {
                            self.pc += 2;
                        }
                    }

                    // 5XY0: skip next if v[X] = v[Y]
                    0x5000 => {
                        println!("Opcode: {:#X} (Skip Reg Equal)", opcode);
                        let r1 = (n2 >> 8) as usize;
                        let r2 = (n3 >> 4) as usize;
                        if self.v[r1] == self.v[r2] {
                            self.pc += 2
                        }
                    }

                    // 6XNN: set register VX to NN
                    0x6000 => {
                        println!("Opcode: {:#X} (Set)", opcode);
                        let r = (n2 >> 8) as usize;
                        let val = (n3 | n4) as u8;
                        self.v[r] = val;
                    }

                    // 7XNN: add NN to value in VX
                    0x7000 => {
                        println!("Opcode: {:#X} (Add)", opcode);
                        let r = (n2 >> 8) as usize;
                        let val = (n3 | n4) as u8;

                        // Prevent overflow panic, wrap instead
                        self.v[r] = self.v[r].wrapping_add(val);
                    }

                    // 8XYN: Logical and arithmetic
                    0x8000 => {
                        let x = (n2 >> 8) as usize;
                        let y = (n3 >> 4) as usize;

                        match n4 {
                            // 0: set v[X] = v[Y]
                            0x0 => {
                                println!("Opcode: {:#X} (Set Register)", opcode);
                                self.v[x] = self.v[y];
                            }

                            // 1: set v[X] = v[X] | v[Y]
                            0x1 => {
                                println!("Opcode: {:#X} (Binary OR)", opcode);
                                self.v[x] = self.v[x] | self.v[y];
                            }

                            // 2: set v[X] = v[X] & v[Y]
                            0x2 => {
                                println!("Opcode: {:#X} (Binary AND)", opcode);
                                self.v[x] = self.v[x] & self.v[y];
                            }

                            // 3: set v[X] = v[X] ^ v[Y]
                            0x3 => {
                                println!("Opcode: {:#X} (XOR)", opcode);
                                self.v[x] = self.v[x] ^ self.v[y];
                            }

                            // 4: set v[X] = v[X] + v[Y], set v[F] to carry
                            0x4 => {
                                println!("Opcode: {:#X} (Add Registers)", opcode);
                                let (sum, overflow) = self.v[x].overflowing_add(self.v[y]);
                                self.v[x] = sum;
                                self.v[0xF] = if overflow { 1 } else { 0 };
                            }

                            // 5: set v[X] = v[X] - v[Y], v[F] = 1 if v[X] > v[Y] else 0
                            0x5 => {
                                println!("Opcode: {:#X} (Subtract X-Y)", opcode);
                                let (res, underflow) = self.v[x].overflowing_sub(self.v[y]);
                                self.v[x] = res;
                                self.v[0xF] = if underflow { 0 } else { 1 };
                            }

                            /*
                                6: right shift v[X]
                                modern: ignore Y
                                original: set v[X] = v[Y] first
                            */
                            0x6 => {
                                println!("Opcode: {:#X} (Right Shift)", opcode);
                                if !self.modern {
                                    self.v[x] = self.v[y]
                                }
                                
                                let old = self.v[x];
                                self.v[x] >>= 1;

                                // Set v[F] to shifted out bit
                                self.v[0xF] = old & 1;
                            }

                            // 7: set v[X] = v[Y] - v[X], v[F] = 1 if v[Y] > v[X] else 0
                            0x7 => {
                                println!("Opcode: {:#X} (Subtract Y-X)", opcode);
                                let (res, underflow) = self.v[y].overflowing_sub(self.v[x]);
                                self.v[x] = res;
                                self.v[0xF] = if underflow { 0 } else { 1 };
                            }

                            /*
                                E: left shift v[X]
                                modern: ignore Y
                                original: set v[X] = v[Y] first
                            */
                            0xE => {
                                println!("Opcode: {:#X} (Left Shift)", opcode);
                                if !self.modern {
                                    self.v[x] = self.v[y]
                                }

                                let old = self.v[x];
                                self.v[x] <<= 1;

                                // Set v[F] to shifted out bit
                                self.v[0xF] = (old >> 7) & 1;
                            }

                            _ => println!("Opcode: {:#X} (not implemented)", opcode)
                        }
                    }

                    // 9XY0: skip next if v[X] != v[Y]
                    0x9000 => {
                        println!("Opcode: {:#X} (Skip Reg Not Equal)", opcode);
                        let r1 = (n2 >> 8) as usize;
                        let r2 = (n3 >> 4) as usize;
                        if self.v[r1] != self.v[r2] {
                            self.pc += 2
                        }
                    }

                    // ANNN: set i register to 0x0NNN
                    0xA000 => {
                        println!("Opcode: {:#X} (Set i)", opcode);
                        self.i = n2 | n3 | n4;
                    }

                    /*
                        BNNN / BXNN: jump with offset
                        modern: jump to XNN + v[X]
                        original: jump to NNN + v[0]
                    */
                    0xB000 => {
                        println!("Opcode: {:#X} (Jump With Offset)", opcode);
                        if self.modern {
                            let xnn = n2 | n3 | n4;
                            let x = (n2 >> 8) as usize;
                            self.pc = xnn + (self.v[x] as u16);
                        } else {
                            let nnn = n2 | n3 | n4;
                            self.pc = nnn + (self.v[0] as u16);
                        }
                    }

                    // CXNN: v[X] = bitwise AND random u8 with NN
                    0xC000 => {
                        println!("Opcode: {:#X} (Random)", opcode);
                        let r: u8 = self.rng.gen_range(0..=255);
                        let nn = (n3 | n4) as u8;
                        let x = (n2 >> 8) as usize;
                        self.v[x] = r & nn;
                    }

                    // DXYN: display
                    0xD000 => {
                        println!("Opcode: {:#X} (Display)", opcode);
                        let x = (n2 >> 8) as usize;
                        let y = (n3 >> 4) as usize;
                        let n = n4 as usize;
                        let vx = self.v[x] as usize;
                        let vy = self.v[y] as usize;
                        self.v[0xF] = 0;

                        for row in 0..n {
                            let spr_byte = self.memory[(self.i as usize) + row];

                            // Iterate bit by bit (MSB to LSB)
                            for i in (0..=7).rev() {
                                let bit = (spr_byte >> i) & 1;
                                if bit == 1 {
                                    let draw_x = (vx + 7 - i) & 63;
                                    let draw_y = (vy + row) & 31;
                                    let display_index = draw_y * 64 + draw_x;
                                    
                                    if self.display[display_index] == 1 {
                                        self.display[display_index] = 0;
                                        self.v[0xF] = 1;
                                    } else {
                                        self.display[display_index] = 1;
                                    }
                                }
                            }
                        }

                        return ExecutionEffect::DisplayUpdate;
                    }

                    // E: Key
                    0xE000 => {
                        let x = (n2 >> 8) as usize;
                        let b2 = n3 | n4;

                        match b2 {
                            // EX9E: skip if keypad[v[X]]
                            0x9E => {
                                println!("Opcode: {:#X} (Skip if Key)", opcode);
                                if self.keypad[self.v[x] as usize] {
                                    self.pc += 2;
                                }
                            }

                            // EXA1: skip if !keypad[v[X]]
                            0xA1 => {
                                println!("Opcode: {:#X} (Skip if not Key)", opcode);
                                if !self.keypad[self.v[x] as usize] {
                                    self.pc += 2;
                                }
                            }

                            _ => println!("Opcode: {:#X} (not implemented)", opcode)
                        }
                    }

                    // F: Misc.
                    0xF000 => {
                        let x = (n2 >> 8) as usize;
                        let b2 = n3 | n4;

                        match b2 {
                            // 07: set v[X] to delay timer
                            0x07 => {
                                println!("Opcode: {:#X} (Set to Delay Timer)", opcode);
                                self.v[x] = self.delay_timer;
                            }

                            // 0A: wait for key, set v[X] to key number on press
                            0x0A => {
                                println!("Opcode: {:#X} (Wait For Key)", opcode);
                                if !self.key_pressed {
                                    self.pc -= 2;
                                    return ExecutionEffect::WaitingForKey;
                                } else {
                                    self.v[x] = self.pressed_key as u8;
                                    self.key_pressed = false;
                                }
                            }

                            // 15: set delay timer = v[X]
                            0x15 => {
                                println!("Opcode: {:#X} (Set Delay Timer)", opcode);
                                self.delay_timer = self.v[x];
                            }

                            // 18: set sound timer = v[X]
                            0x18 => {
                                println!("Opcode: {:#X} (Set Sound Timer)", opcode);
                                self.sound_timer = self.v[x];
                                if self.sound_timer > 0 {
                                    return ExecutionEffect::Sound;
                                }
                            }

                            // 1E: add v[X] to i
                            0x1E => {
                                println!("Opcode: {:#X} (Add i)", opcode);
                                let (res, overflow) = self.i.overflowing_add(self.v[x] as u16);
                                self.i = res;
                                self.v[0xF] = if overflow { 1 } else { 0 };
                            }

                            // 29: set i to location of font character in v[X]
                            0x29 => {
                                println!("Opcode: {:#X} (Font Character)", opcode);
                                self.i = (self.v[x] * 5) as u16;
                            }

                            // 33: store decimal digits of number in v[X] in memory
                            0x33 => {
                                println!("Opcode: {:#X} (Decimal)", opcode);
                                let h = self.v[x] / 100;
                                let t = (self.v[x] - h * 100) / 10;
                                let u = self.v[x] - h * 100 - t * 10;

                                self.memory[self.i as usize] = h;
                                self.memory[(self.i + 1) as usize] = t;
                                self.memory[(self.i + 2) as usize] = u;
                            }

                            /*
                                55: store registers (v[0] up to v[X]) in memory
                                modern: leave i unchanged
                                original: update i to i + X + 1
                            */
                            0x55 => {
                                println!("Opcode: {:#X} (Store Registers)", opcode);
                                for i in 0..=x {
                                    let mem_index = (self.i as usize) + i;
                                    self.memory[mem_index] = self.v[i];
                                }

                                if !self.modern {
                                    self.i += (x + 1) as u16;
                                }
                            }

                            /*
                                65: load registers (v[0] up to v[X]) from memory
                                modern: leave i unchanged
                                original: update i to i + X + 1
                            */
                            0x65 => {
                                println!("Opcode: {:#X} (Load Registers)", opcode);
                                for i in 0..=x {
                                    let mem_index = (self.i as usize) + i;
                                    self.v[i] = self.memory[mem_index];
                                }

                                if !self.modern {
                                    self.i += (x + 1) as u16;
                                }
                            }

                            _ => println!("Opcode: {:#X} (not implemented)", opcode)
                        }
                    }

                    _ => println!("Opcode: {:#X} (not implemented)", opcode)
                }
            }
        }

        ExecutionEffect::NoEffect
    }
}