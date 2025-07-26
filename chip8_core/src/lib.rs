use rand::Rng;

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

    fn push(&mut self, val: u16){
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16{
        self.sp -=1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self){
        //FETCH
        let op = self.fetch();
        //DECODE & EXECUTE
        self.execute(op)
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, index: usize, pressed: bool){
        self.keys[index] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
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
                let jump_addr = op & 0xFFF;                
                self.push(self.pc);
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
            (5, _, _, _) => { //5XY0 Skip if Vx reg = Vy reg
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
                let val = op & 0xFFF;
                self.i_reg = val;
            }
            (0xB, _, _, _) => { //BNNN Jump to V0 + 0xNNN
                let distance = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + distance;
            }
            (0xC, _, _, _) => { //CXNN Vx = rand() & NN
                let regx = digit2 as usize;
                let val = (op & 0xFF) as u8;
                let rng: u8 = rand::thread_rng().r#gen();
                self.v_reg[regx] = rng & val;
            }
            (0xD, _, _, _) => //DXYN Draw sprite at (Vx, VY). 
            // Sprite is 0xN pixels tall
            // On/off based on value in I register
            // Vf set if any pixels are flipped
            {
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;

                // The last digit determines how many rows high our sprite is
                let num_rows = digit4;

                let mut flipped = false;

                for y_line in 0..num_rows{
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8  {//Chip-8's sprites are always 8 pixels wide
                        // Use a mask to fetch current pixel's bit. Only flip if a 1.k
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            //Sprites wrap around the screen, apply modulo operation
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            //Get pixel index for 1 Dimensional Screen Array
                            let index = x + SCREEN_WIDTH * y;

                            //Check if the pixel is going to be flipped to set bool
                            flipped |= self.screen[index];
                            self.screen[index] ^= true;
                        }
                    }
                }

                //Populate VF register
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            },
            (0xE, _, 9, 0xE) => { //EX9E Skip if Key (index in Vx register) is Pressed
                let x = digit2 as usize;
                let reg = self.v_reg[x];
                let key = self.keys[reg as usize];

                if key {self.pc += 2};
            }
            (0xE, _, 0xA, 1) => { //EXA1 Skip if key not pressed
                let x = digit2 as usize;
                let reg = self.v_reg[x];
                let key = self.keys[reg as usize];

                if !key {self.pc +=2};
            }
            (0xF, _, 0, 7) => { //FX07 Set Vx to DT
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            }
            (0xF, _, 0, 0xA) => { //FX0A Wait for key press (key index in Vx reg)
                let x = digit2 as usize;
                let mut pressed = false;
                
                for i in 0..self.keys.len(){
                    if self.keys[i]{
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                //Redo opcode
                if !pressed {self.pc -= 2};
            }
            (0xF, _, 1, 5) => { //FX15 Set DT to Vx
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            }
            (0xF, _, 1, 8) => { //FX18 Set ST to Vx
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            }
            (0xF, _, 1, 0xE) => { //FX1E Set I to I + VX, handle rollover
                let x = digit2 as usize;
                let reg = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(reg);
            }
            (0xF, _, 2, 9) => { //FX29 Set I to Font Address
                let x = digit2 as usize;
                let char = self.v_reg[x] as u16;
                self.i_reg = char *  5;
            }
            (0xF, _, 3, 3) => { //FX33 I = BCD of Vx, Used to display the decimal of a stored binary/hex number
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;

                // Fetch hundreds digit: Divide by 100, toss the decimal
                let hundreds = (vx / 100.0).floor() as u8;
                // Fetch the tens digit: Divide by 10, toss the ones digit and the decimal
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                // Fetch the ones digit by tossing the hundreds and the tens
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }
            (0xF, _, 5, 5) => { //FX55 Store V0 THROUGH Vx into I
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for index in 0..=x {
                    self.ram[i + index] = self.v_reg[index];
                }
            }
            (0xF, _, 6, 5) => { //FX65 Load I into V0 through Vx
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for index in 0..=x {
                    self.v_reg[index] = self.ram[i + index];
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }
}