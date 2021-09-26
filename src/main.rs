mod assembler;

use assembler::{Instruction, InstructionCommand, InstructionArgument};

fn main() {
    let mut cpu = initialize_cpu();

    // set A to 12, to see effects of assembly code execution
    cpu.set_register(0, initialize_register(12));
    
    let assembler = assembler::Assembler::new("test.asm".to_owned(), "output".to_owned());

    assembler.assemble();
    let instructions = assembler.disassemble("output".to_owned());
    cpu.run(&instructions);
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

    fn run(&mut self, instructions: &Vec<Instruction>) {
        println!("Initial status:");
        self.get_status();

        for instruction in instructions {
            println!("-------------");
            println!("{:?}", instruction);

            self.execute(instruction);
            self.get_status();
        }
    }

    fn execute(&mut self, instruction: &Instruction) {
        match instruction.command {
            InstructionCommand::MOV => self.execute_mov(&instruction.arguments),
            InstructionCommand::ADD => self.execute_add(&instruction.arguments),
            InstructionCommand::SUB => self.execute_sub(&instruction.arguments),
            InstructionCommand::INR => self.execute_inr(&instruction.arguments),
            InstructionCommand::DCR => self.execute_dcr(&instruction.arguments),
            InstructionCommand::HLT => self.execute_hlt(&instruction.arguments),
            _ => panic!("Invalid command!")
        }        
    }

    fn execute_add(&mut self, args: &Vec<InstructionArgument>) {
        let source_register = self.get_register(args[0].to_index().into());        
        let current_a = self.get_register(0).value;

        let added_register = Register {value: current_a+source_register.value};
        self.set_register(0, added_register);
    }

    fn execute_sub(&mut self, args: &Vec<InstructionArgument>) {
        let source_register = self.get_register(args[0].to_index().into());        
        let current_a = self.get_register(0).value;

        let subtracted_register = Register {value: current_a-source_register.value};
        self.set_register(0, subtracted_register);
    }

    fn execute_mov(&mut self, args: &Vec<InstructionArgument>) {
        let source_register = self.get_register(args[0].to_index().into());        
        let new_register = Register {value: source_register.value};

        let destination_index = args[1].to_index().into();
        self.set_register(destination_index, new_register);
    }

    fn execute_inr(&mut self, args: &Vec<InstructionArgument>) {
        let register = self.get_register(args[0].to_index().into());        

        let incremented_register = Register {value: register.value+1};

        self.set_register(args[0].to_index().into(), incremented_register);
    }

    fn execute_dcr(&mut self, args: &Vec<InstructionArgument>) {
        let register = self.get_register(args[0].to_index().into());        

        let incremented_register = Register {value: register.value-1};

        self.set_register(args[0].to_index().into(), incremented_register);
    }

    fn execute_hlt(&mut self, args: &Vec<InstructionArgument>) {
        println!("Execution finished");
        println!("Final status: ");
        self.get_status();
        std::process::exit(0);
    }

    fn get_status(&self) {
        for i in 0..7 {
            println!("{:?}: {:#08b}", i, self.get_register(i).value);
        }
    }
}