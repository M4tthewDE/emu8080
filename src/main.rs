mod assembler;

fn main() {
    let mut cpu = initialize_cpu();
    println!("{:?}", cpu.get_register(0));

    cpu.set_register(0, initialize_register(12));
    
    println!("{:?}", cpu.get_register(0));

    let assembler = assembler::Assembler::new("test.asm".to_owned(), "output".to_owned());
    assembler.assemble();
}

fn initialize_cpu() -> Cpu {
    let mut registers = Vec::new();
    for _ in 0..7 {
        registers.push(initialize_register(0));
    }

    Cpu {
        register: registers
    }
}

fn initialize_register(value: u8) -> Register {
    Register {
        value: value
    }
}

#[derive(Debug)]
struct Cpu {
    register: Vec<Register>
}

#[derive(Debug)]
struct Register {
   value: u8 
}

impl Cpu {
    fn get_register(&self, index: usize) -> &Register {
        &self.register[index]
    }

    fn set_register(&mut self, index: usize, register: Register) {
        self.register[index] = register;
    }
}