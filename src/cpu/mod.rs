use crate::assembler::{Instruction, InstructionCommand, InstructionArgument};

pub fn initialize_cpu() -> Cpu {
    let mut registers = Vec::new();
    for _ in 0..7 {
        registers.push(initialize_register(0));
    }

    Cpu {
        register: registers
    }
}

pub fn initialize_register(value: u8) -> Register {
    Register {
        value: value
    }
}

#[derive(Debug)]
pub struct Cpu {
    register: Vec<Register>
}

#[derive(Debug)]
pub struct Register {
   value: u8 
}

impl Cpu {
    fn get_register(&self, index: usize) -> &Register {
        &self.register[index]
    }

    pub fn set_register(&mut self, index: usize, register: Register) {
        self.register[index] = register;
    }

    fn change_register(&mut self, index: usize, value: u8) {
        self.register[index].value = value;
    }

    pub fn run(&mut self, instructions: &Vec<Instruction>) {
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
            InstructionCommand::HLT => self.execute_hlt(),
        }        
    }

    fn execute_mov(&mut self, args: &Vec<InstructionArgument>) {
        let source_value = self.get_register(args[0].to_index().into()).value;        

        let destination_index = args[1].to_index().into();
        self.change_register(destination_index, source_value);
    }

    fn execute_add(&mut self, args: &Vec<InstructionArgument>) {
        let source_value = self.get_register(args[0].to_index().into()).value;        
        let current_a = self.get_register(0).value;

        self.change_register(0, current_a+source_value);
    }

    fn execute_sub(&mut self, args: &Vec<InstructionArgument>) {
        let source_value = self.get_register(args[0].to_index().into()).value;        
        let current_a = self.get_register(0).value;

        self.change_register(0, current_a-source_value);
    }

    fn execute_inr(&mut self, args: &Vec<InstructionArgument>) {
        let value = self.get_register(args[0].to_index().into()).value;        

        self.change_register(args[0].to_index().into(), value+1);
    }

    fn execute_dcr(&mut self, args: &Vec<InstructionArgument>) {
        let value = self.get_register(args[0].to_index().into()).value;        

        self.change_register(args[0].to_index().into(), value-1);
    }

    fn execute_hlt(&mut self) {
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