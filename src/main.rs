use bitvec::prelude::*;

fn main() {
    let mut cpu = initialize_cpu();
    println!("{:?}", cpu.get_register(0));

    cpu.set_register(0, initialize_register(12));
    
    println!("{:?}", cpu.get_register(0));
}

fn initialize_cpu() -> Cpu {
    let mut registers = Vec::new();
    for _ in 0..8 {
        registers.push(initialize_register(0));
    }

    Cpu {
        register: registers
    }
}

fn initialize_register(value: u8) -> Register {
    Register {
        bits: BitVec::from_element(value)
    }
}

#[derive(Debug)]
struct Cpu {
    register: Vec<Register>
}

#[derive(Debug)]
struct Register {
   bits: BitVec<Msb0, u8> 
}

impl Cpu {
    fn get_register(&self, index: usize) -> &Register {
        &self.register[index]
    }

    fn set_register(&mut self, index: usize, register: Register) {
        self.register[index] = register;
    }
}