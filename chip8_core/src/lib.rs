use rand::random;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const START_ADDR: u16 = 0x200;

pub struct Emu {
    pc: u16, //Program Counter
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16, //Stack Pointer
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8, //Delay Timer
    st: u8, //Sound Timer
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self){
        //FETCH
        let op = self.fetch();
        //DECODE & EXECUTE
        self.execute(op)
    }

    pub fn tick_timers(&mut self){
        if self.dt > 0{
            self.dt -= 1;
        }

        if self.st > 0{
            if self.st == 1 {
                //BEEP
            }
            self.st -= 1;
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte= self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
            self.pc += 2;
            op
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return, //0000 No operation, proceed;
            (0, 0, 0xE, 0) => { //00E0 Clear Screen 
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            (0, 0, 0xE, 0xE) => { //00EE Return from Subroutine
                let return_addr = self.pop();
                self.pc = return_addr;
            },
            (1, _, _, _) => { //1NNN Jump to Address 0xNNN
                let jump_addr = op & 0xFFF;
                self.pc = jump_addr;
            },
            (2, _, _, _) => { //2NNN Enter subroutine 0xNNN
                self.push(self.pc);

                let jump_addr = op & 0xFFF;
                self.pc = jump_addr;
            },
            (3, _, _, _) => { //3XNN Skip if Vx register == 0xNN
                let reg = digit2 as usize;
                let val = (op & 0xFF) as u8;
                if self.v_reg[reg] == val {
                    self.pc += 2; //Skip forward one opcode
                }
            },
            (4, _, _, _) => { //4XNN Skip if register != 0xNN
                let reg = digit2 as usize;
                let val = (op & 0xFF) as u8;
                if self.v_reg[reg] != val {
                    self.pc += 2; //Skip forward one opcode
                }
            },
            (5, _, _, 0) => { //5XY0 Skip if Vx reg = Vy reg
                let regx = digit2 as usize;
                let regy = digit3 as usize;
                if self.v_reg[regx] == self.v_reg[regy]{
                    self.pc += 2;
                }

            },
            (6, _, _, _) => { //6XNN Set Vx to NN
                let reg = digit2 as usize;
                let val = (op & 0xFF) as u8;
                self.v_reg[reg] = val;
            },
            (7, _, _, _) => { //7XNN Vx += NN
                let reg = digit2 as usize;
                let value = (op & 0xFF) as u8;
                self.v_reg[reg] = self.v_reg[reg].wrapping_add(value);
            },
            (8, _, _, 0) => { //8XY0 Vx = Vy
                let regx = digit2 as usize;
                let regy = digit3 as usize;
                self.v_reg[regx] = self.v_reg[regy];
            },
            (8, _, _, 1) => { //8XY1 Vx |= Vy
                let regx = digit2 as usize;
                let regy = digit3 as usize;
                self.v_reg[regx] |= self.v_reg[regy]
            },
            (8, _, _, 2) => { //8XY2 Vx &= Vy
                let regx = digit2 as usize;
                let regy = digit3 as usize;
                self.v_reg[regx] &= self.v_reg[regy];
            },
            (8, _, _, 3) => { //8XY3 Vx ^= Vy
                let regx = digit2 as usize;
                let regy = digit3 as usize;
                self.v_reg[regx] ^= self.v_reg[regy];
            }
            (8, _, _, 4) => { //8XY4 Vx += VY, Sets Vf if carry
                let regx = digit2 as usize;
                let regy = digit3 as usize;
                let (new_regx, carry) = self.v_reg[regx].overflowing_add(self.v_reg[regy]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[regx] = new_regx;
                self.v_reg[0xF] = new_vf;
            }
            (8, _, _, 5) => { //8XY5 Vx -= Vy, Clears Vf if borrow
                let regx = digit2 as usize;
                let regy = digit3 as usize;
                let (new_regx, borrow) = self.v_reg[regx].overflowing_sub(self.v_reg[regy]);
                let new_vf = if borrow { 0 }  else { 1 };

                self.v_reg[regx] = new_regx;
                self.v_reg[0xF] = new_vf;
            }
            (8, _, _, 6) => { //8XY6 Vx >>= 1, Store dropped (least significant) bit in Vf
                let regx = digit2 as usize;
                let lsb = self.v_reg[regx] & 1;

                self.v_reg[regx] >>= 1;
                self.v_reg[0xF] = lsb;
            }
            (8, _, _, 7) => { //8XY7 Vx = Vy - Vx, Clears Vf if borrow
                let regx = digit2 as usize;
                let regy = digit3 as usize;
                let (new_regx, borrow) = self.v_reg[regy].overflowing_sub(self.v_reg[regx]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[regx] = new_regx;
                self.v_reg[0xF] = new_vf;
            }
            (8, _, _, 0xE) => { //8XYE Vx <<= 1, Stored dropped (most significant) bit in Vf
                let regx = digit2 as usize;
                let msb = (self.v_reg[regx] >> 7) & 1;

                self.v_reg[regx] <<= 1;
                self.v_reg[0xF] = msb;
            }
            (9, _, _, 0) => { //9XY0 Skip if Vx != Vy
                let regx = digit2 as usize;
                let regy = digit3 as usize;
                if self.v_reg[regx] != self.v_reg[regy]{
                    self.pc += 2;
                }

            },
            (0xA, _, _, _) => { //ANNN I reg = NNN
                self.i_reg = op & 0xFFF;
            }
            (0xB, _, _, _) => { //BNNN Jump to V0 + 0xNNN
                let distance = op & 0xFFF;
                let target = (self.v_reg[0] as u16) + distance;
                self.pc = target;
            }
            (0xC, _, _, _) => { //CXNN Vx = rand() & NN
                let regx = digit2 as usize;
                let val = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[regx] = rng & val;
            }
            (0xD, _, _, _) => //DXYN Draw sprite at (Vx, VY). 
            // Sprite is 0xN pixels tall
            // On/off based on value in I register
            // Vf set if any pixels are flipped
            {
                let x_coord = self.v_reg[digit2 as usize];
                let y_coord = self.v_reg[digit3 as usize];
                let num_rows = digit4;

                let mut flipped = false;

                for y_line in 0..num_rows{
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8  {//Chip-8's sprites are always 8 pixels wide
                        // Use a mask to fetch current pixel's bit. Only flip if a 1.
                    }
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }
    
    fn push(&mut self, val: u16){
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16{
        self.sp -=1;
        self.stack[self.sp as usize]
    }
}