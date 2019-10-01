

const NUM_REG: usize = 8;

type Reg = u16;
type Word = u16;

struct Processor {
    sp: Reg,
    pc: Reg,

    zero: bool,

    reg_file: [Reg; NUM_REG],

    mem: Vec<Word>
}

impl Processor {
    pub fn new(amount_mem: usize) -> Processor {
        return Processor {
            sp: 0,
            pc: 0,

            zero: false,

            reg_file: [0; NUM_REG],

            mem: Vec::with_capacity(amount_mem),
        };
    }

    pub fn step(&mut self, instr: Instr) {
        use Instr::*;

        match instr {
            Add(reg, reg2) => {
                self.reg_file[reg as usize] += self.reg_file[reg2 as usize];
            },

            Sub(reg, reg2) => {
                self.reg_file[reg as usize] -= self.reg_file[reg2 as usize];
            },

            Mul(reg, reg2) => {
                self.reg_file[reg as usize] *= self.reg_file[reg2 as usize];
            },

            Div(reg, reg2) => {
                self.reg_file[reg as usize] /= self.reg_file[reg2 as usize];
            },

            Mov(reg, reg2) => {
                self.reg_file[reg as usize] = self.reg_file[reg2 as usize];
            },

            Store(reg, reg2) => {
                self.mem[reg as usize] = self.reg_file[reg2 as usize];
            },

            Load(reg, reg2) => {
                self.reg_file[reg as usize] = self.mem[reg2 as usize];
            },

            Push(reg) => {
                self.mem[self.sp as usize] = self.reg_file[reg as usize];
                self.sp += 1;
            },

            Pop(reg) => {
                self.sp -= 1;
                self.reg_file[reg as usize] = self.mem[self.sp as usize];
            },

            Jmp(loc) => {
                self.pc = loc;
            },

            JmpZ(loc) => {
                if self.zero {
                    self.pc = loc;
                }
            },

            JmpNZ(loc) => {
                if !self.zero {
                    self.pc = loc;
                }
            },

            JmpRel(offset) => {
                    self.pc += offset;
            },

            JmpZRel(offset) => {
                if self.zero {
                    self.pc += offset;
                }
            },

            JmpNZRel(offset) => {
                if !self.zero {
                    self.pc += offset;
                }
            },

        }
    }
}

enum Instr {
    Add(Reg, Reg),
    Sub(Reg, Reg),
    Mul(Reg, Reg),
    Div(Reg, Reg),
    Mov(Reg, Reg),
    Store(Reg, Reg),
    Load(Reg, Reg),
    Push(Reg),
    Pop(Reg),
    Jmp(Word),
    JmpZ(Word),
    JmpNZ(Word),
    JmpRel(Word),
    JmpZRel(Word),
    JmpNZRel(Word),
}



fn main() {
    println!("Hello, world!");
}
