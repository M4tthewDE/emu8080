use crate::assembler::{Instruction, InstructionCommand, InstructionRegister};
use std::convert::TryFrom;

pub fn initialize_cpu() -> Cpu {
    let mut registers = Vec::new();
    for _ in 0..7 {
        registers.push(0);
    }

    Cpu {
        register: registers,
        flags: vec![0,0,0,0,0,0,0,0],
    }
}

#[derive(Debug)]
pub struct Cpu {
    register: Vec<u8>,

    // S Z x A x P x C
    flags: Vec<u8>,
}

impl Cpu {
    fn get_register(&self, index: usize) -> &u8{
        &self.register[index]
    }

    fn change_register(&mut self, index: usize, value: u8) {
        self.register[index] = value;
    }

    pub fn run(&mut self, instructions: &Vec<Instruction>) {
        println!("Initial status:");
        self.print_status();

        for instruction in instructions {
            println!("-------------");
            println!("{:?}", instruction);

            self.execute(instruction);
            self.print_status();
        }
    }

    fn execute(&mut self, instruction: &Instruction) {
        match instruction.command {
            InstructionCommand::MVI => self.execute_mvi(&instruction.registers, &instruction.intermediate),
            InstructionCommand::MOV => self.execute_mov(&instruction.registers),
            InstructionCommand::ADD => self.execute_add(&instruction.registers),
            InstructionCommand::SUB => self.execute_sub(&instruction.registers),
            InstructionCommand::INR => self.execute_inr(&instruction.registers),
            InstructionCommand::DCR => self.execute_dcr(&instruction.registers),
            InstructionCommand::HLT => self.execute_hlt(),
        }        
    }

    fn execute_mvi(&mut self, args: &Vec<InstructionRegister>, intermediate: &Vec<u8>) {
        let destination_index = args[0].to_index().into();

        let mut value = 0;
        for (index, digit) in intermediate.iter().rev().enumerate() {
            value = value + (digit*u8::pow(2, u32::try_from(index).unwrap()));
        }
       self.change_register(destination_index, value); 
    }

    fn execute_mov(&mut self, args: &Vec<InstructionRegister>) {
        let source_value = *self.get_register(args[0].to_index().into());        

        let destination_index = args[1].to_index().into();
        self.change_register(destination_index, source_value);
    }

    fn execute_add(&mut self, args: &Vec<InstructionRegister>) {
        let source_value = self.get_register(args[0].to_index().into());        
        let current_a = self.get_register(0);
        let new_a = current_a+source_value;

        self.change_register(0, new_a);

        if self.get_register(0) == &0 {
            self.set_zero_flag(1);
        } else {
            self.set_zero_flag(0);
        }
    }

    fn execute_sub(&mut self, args: &Vec<InstructionRegister>) {
        let source_value = self.get_register(args[0].to_index().into());        
        let current_a = self.get_register(0);
        let new_a = current_a-source_value;

        self.change_register(0, new_a);

        if self.get_register(0) == &0 {
            self.set_zero_flag(1);
        } else {
            self.set_zero_flag(0);
        }
    }

    fn execute_inr(&mut self, args: &Vec<InstructionRegister>) {
        let new_value = self.get_register(args[0].to_index().into())+1;        

        self.change_register(args[0].to_index().into(), new_value);

        if self.get_register(0) == &0 {
            self.set_zero_flag(1);
        } else {
            self.set_zero_flag(0);
        }
    }

    fn execute_dcr(&mut self, args: &Vec<InstructionRegister>) {
        let new_value = self.get_register(args[0].to_index().into())-1;        

        self.change_register(args[0].to_index().into(), new_value);

        if self.get_register(0) == &0 {
            self.set_zero_flag(1);
        } else {
            self.set_zero_flag(0);
        }
    }

    fn set_zero_flag(&mut self, value: u8) {
       self.flags[1] = value; 
    }

    fn execute_hlt(&mut self) {
        println!("Execution finished");
        println!("Final status: ");
        self.print_status();
        std::process::exit(0);
    }

    fn print_status(&self) {
        for i in 0..7 {
            println!("{:?}: {:#010b}", i, self.get_register(i));
        }
        self.print_flags();
    }

    fn print_flags(&self) {
        println!("Flags:");

        let flags = vec!['S', 'Z', 'x', 'A', 'x', 'P', 'x', 'C'];
        for (i, flag) in self.flags.iter().enumerate() {
            if i != 2 && i != 4 && i != 6 {
                println!("{}: {:?}", flags[i], flag);
            }
        }
    }
}