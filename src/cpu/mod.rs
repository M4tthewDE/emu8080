use crate::assembler::{Instruction, InstructionCommand, InstructionRegister};
use std::convert::TryFrom;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub fn initialize_cpu() -> Cpu {
    Cpu {
        register: vec![0,0,0,0,0,0,0,0],
        flags: vec![0,0,0,0,0,0,0,0],
    }
}

#[derive(Debug)]
pub struct Cpu {
    register: Vec<i8>,

    // S Z x A x P x C
    flags: Vec<u8>,
}

impl Cpu {
    fn get_register(&self, index: usize) -> &i8{
        &self.register[index]
    }

    fn change_register(&mut self, index: usize, value: i8) {
        self.register[index] = value;
    }

    pub fn run(&mut self, instructions: &[Instruction]) {
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
            InstructionCommand::MVI => self.execute_mvi(&instruction.registers[0], &instruction.intermediate),
            InstructionCommand::ADI => self.execute_adi(&instruction.intermediate),
            InstructionCommand::MOV => self.execute_mov(&instruction.registers),
            InstructionCommand::ADD => self.execute_add(&instruction.registers[0]),
            InstructionCommand::SUB => self.execute_sub(&instruction.registers[0]),
            InstructionCommand::INR => self.execute_inr(&instruction.registers[0]),
            InstructionCommand::DCR => self.execute_dcr(&instruction.registers[0]),
            InstructionCommand::ANA => self.execute_ana(&instruction.registers[0]),
            InstructionCommand::HLT => self.execute_hlt(),
        }        
    }

    fn execute_mvi(&mut self, arg: &InstructionRegister, intermediate: &[u8]) {
        let destination_index = arg.to_index().into();

        let mut x = vec![0; 8];
        x[0..].clone_from_slice(intermediate);

        let value = self.binary_to_int(&mut x); 

        self.change_register(destination_index, value); 
    }

    fn execute_adi(&mut self, intermediate: &[u8]) {
        let mut x = vec![0; 8];
        x[0..].clone_from_slice(intermediate);

        let source_value = self.binary_to_int(&mut x);        
        let current_a = self.get_register(0);
        let new_a = current_a+source_value;

        self.change_register(0, new_a);

        if self.get_register(0) == &0 {
            self.set_flag(Flag::Z, 1);
        } else {
            self.set_flag(Flag::Z, 0);
        }

        if self.get_register(0) < &0 {
            self.set_flag(Flag::S, 1);
        } else {
            self.set_flag(Flag::S, 0);
        }
    }

    fn execute_mov(&mut self, args: &[InstructionRegister]) {
        let source_value = *self.get_register(args[0].to_index().into());        

        let destination_index = args[1].to_index().into();
        self.change_register(destination_index, source_value);
    }

    fn execute_add(&mut self, arg: &InstructionRegister) {
        let source_value = self.get_register(arg.to_index().into());        
        let current_a = self.get_register(0);
        let new_a = current_a+source_value;

        self.change_register(0, new_a);

        if self.get_register(0) == &0 {
            self.set_flag(Flag::Z, 1);
        } else {
            self.set_flag(Flag::Z, 0);
        }

        if self.get_register(0) < &0 {
            self.set_flag(Flag::S, 1);
        } else {
            self.set_flag(Flag::S, 0);
        }
    }

    fn execute_sub(&mut self, args: &InstructionRegister) {
        let source_value = self.get_register(args.to_index().into());        
        let current_a = self.get_register(0);
        let new_a = current_a-source_value;

        self.change_register(0, new_a);

        if self.get_register(0) == &0 {
            self.set_flag(Flag::Z, 1);
        } else {
            self.set_flag(Flag::Z, 0);
        }

        if self.get_register(0) < &0 {
            self.set_flag(Flag::S, 1);
        } else {
            self.set_flag(Flag::S, 0);
        }
    }

    fn execute_inr(&mut self, arg: &InstructionRegister) {
        let new_value = self.get_register(arg.to_index().into())+1;        

        self.change_register(arg.to_index().into(), new_value);

        if self.get_register(0) == &0 {
            self.set_flag(Flag::Z, 1);
        } else {
            self.set_flag(Flag::Z, 0);
        }

        if self.get_register(0) < &0 {
            self.set_flag(Flag::S, 1);
        } else {
            self.set_flag(Flag::S, 0);
        }
    }

    fn execute_dcr(&mut self, arg: &InstructionRegister) {
        let new_value = self.get_register(arg.to_index().into())-1;        

        self.change_register(arg.to_index().into(), new_value);

        if self.get_register(0) == &0 {
            self.set_flag(Flag::Z, 1);
        } else {
            self.set_flag(Flag::Z, 0);
        }

        if self.get_register(0) < &0 {
            self.set_flag(Flag::S, 1);
        } else {
            self.set_flag(Flag::S, 0);
        }
    }

    fn execute_ana(&mut self, arg: &InstructionRegister) {
        let mut binary_a = self.int_to_binary(*self.get_register(0));
        let binary_reg = self.int_to_binary(*self.get_register(arg.to_index().into()));

        for index in 0..8 {
            if !(binary_a[index] == 1 && binary_reg[index] == 1) {
                binary_a[index] = 0;
            }
        }

        let mut value = 0;
        if binary_a[0] == 1 {
            value = self.binary_to_int(&mut binary_a);
        } else {
            for (index, digit) in binary_a.iter().rev().enumerate() {
                value += (digit*u8::pow(2, u32::try_from(index).unwrap())) as i8;
            }
        }
        self.change_register(0, value as i8);
    }

    fn set_flag(&mut self, flag: Flag, value: u8) {
       self.flags[flag.get_index()] = value; 
    }

    fn get_flag(&self, flag: Flag) -> u8 {
        self.flags[flag.get_index()]
    }

    fn execute_hlt(&mut self) {
        println!("Execution finished");
        println!("Final status: ");
        self.print_status();
        std::process::exit(0);
    }

    fn binary_to_int(&self, intermediate: &mut [u8]) -> i8 {
        if intermediate[0] == 1 {
            // subtract 1 from intermediate
            let mut index = intermediate.len()-1;
            while index > 0  {
                if intermediate[index] == 1 {
                    intermediate[index] = 0;
                    break;
                } else {
                    intermediate[index] = 1;
                }
                index -= 1;
            } 

            // build complement
            index = 0;
            while index < intermediate.len() {
                if intermediate[index] == 0 {
                    intermediate[index] = 1;
                } else {
                    intermediate[index] = 0;
                }
                index += 1;
            }

            // calculate binary to decimal
            let mut value = 0;
            for (index, digit) in intermediate.iter().rev().enumerate() {
                value += digit*u8::pow(2, u32::try_from(index).unwrap());
            }
            -(value as i8)
        } else {
            let mut value = 0;
            for (index, digit) in intermediate.iter().rev().enumerate() {
                value += digit*u8::pow(2, u32::try_from(index).unwrap());
            }
            value as i8
        }
    }

    fn int_to_binary(&self, value: i8) -> Vec<u8> {
        let binary_string = format!("{:08b}", value);

        let mut result = Vec::new();
        for c in binary_string.chars() {
            result.push((c as u8)-48);
        }
        result
    }

    fn print_status(&self) {
        for i in 0..7 {
            println!("{}: {:#010b} ({})", i, self.get_register(i), self.get_register(i));
        }
        self.print_flags();
    }

    fn print_flags(&self) {
        println!("Flags:");
        for flag in Flag::iter() {
            println!("{:?}: {}", flag.clone(), self.get_flag(flag));
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, EnumIter, Clone)]
enum Flag {
    S,
    Z,
    A,
    P,
    C,
}

impl Flag {
    pub fn get_index(&self) -> usize {
        match self {
            Flag::S => 0,
            Flag::Z => 1,
            Flag::A => 3,
            Flag::P => 5,
            Flag::C => 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::initialize_cpu;
    use crate::cpu::{InstructionRegister, Flag};

    #[test]
    fn test_execute_mvi() {
        let mut cpu = initialize_cpu();

        cpu.execute_mvi(&InstructionRegister::A, &[0,0,0,0,1,1,1,0]);
        assert_eq!(cpu.get_register(0), &14);
    }

    #[test]
    fn test_execute_mov() {
        let mut cpu = initialize_cpu();
        cpu.change_register(0, 10);

        cpu.execute_mov(&[InstructionRegister::A, InstructionRegister::B]);
        assert_eq!(cpu.get_register(1), &10);
    }

    #[test]
    fn test_execute_add() {
        let mut cpu = initialize_cpu();
        cpu.change_register(0, 5);
        cpu.set_flag(Flag::Z, 1);

        cpu.execute_add(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), &10);
        assert_eq!(cpu.get_flag(Flag::Z), 0);

        cpu.change_register(0, -5);
        cpu.execute_add(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), &-10);
        assert_eq!(cpu.get_flag(Flag::S), 1);
    }

    #[test]
    fn test_execute_adi() {
        let mut cpu = initialize_cpu();
        cpu.change_register(0, 5);
        cpu.set_flag(Flag::Z, 1);

        cpu.execute_adi(&[0,0,0,0,0,1,0,1]);
        assert_eq!(cpu.get_register(0), &10);
        assert_eq!(cpu.get_flag(Flag::Z), 0);

        cpu.change_register(0, -5);
        cpu.execute_adi(&[1,1,1,1,1,0,1,1]);
        assert_eq!(cpu.get_register(0), &-10);
        assert_eq!(cpu.get_flag(Flag::S), 1);
    }

    #[test]
    fn test_execute_sub() {
        let mut cpu = initialize_cpu();
        cpu.change_register(0, 5);

        cpu.execute_sub(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), &0);
        assert_eq!(cpu.get_flag(Flag::Z), 1);

        cpu.change_register(0, -5);
        cpu.execute_sub(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), &0);
    }

    #[test]
    fn test_execute_inr() {
        let mut cpu = initialize_cpu();

        cpu.execute_inr(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), &1);

        cpu.change_register(0, -2);
        cpu.execute_inr(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), &-1);
        assert_eq!(cpu.get_flag(Flag::S), 1);
    }

    #[test]
    fn test_execute_dcr() {
        let mut cpu = initialize_cpu();
        cpu.change_register(0, 1);

        cpu.execute_dcr(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), &0);
        assert_eq!(cpu.get_flag(Flag::Z), 1);

        cpu.change_register(0, -1);

        cpu.execute_dcr(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), &-2);
        assert_eq!(cpu.get_flag(Flag::S), 1);
    }

    #[test]
    fn test_execute_ana() {
        let mut cpu = initialize_cpu();
        cpu.change_register(0, -10);
        cpu.change_register(1, -10);

        cpu.execute_ana(&InstructionRegister::B);
        assert_eq!(cpu.get_register(0), &-10);

        // -15 11110001	
        // -10 11110110
        // ANA 11110000

        cpu.change_register(0, -15);
        cpu.execute_ana(&InstructionRegister::B);
        assert_eq!(cpu.get_register(0), &-16);
    }

    #[test]
    fn test_flag_get_index() {
        assert_eq!(Flag::S.get_index(), 0);
        assert_eq!(Flag::Z.get_index(), 1);
        assert_eq!(Flag::A.get_index(), 3);
        assert_eq!(Flag::P.get_index(), 5);
        assert_eq!(Flag::C.get_index(), 7);
    }

    #[test]
    fn test_binary_to_int() {
        let cpu = initialize_cpu();

        let mut intermediate = [1,1,1,1,0,0,0,1];
        let result = cpu.binary_to_int(&mut intermediate);
        assert_eq!(result, -15);

        let mut intermediate = [0,1,1,1,0,0,0,1];
        let result = cpu.binary_to_int(&mut intermediate);
        assert_eq!(result, 113);
    }

    #[test]
    fn test_int_to_binary() {
        let cpu = initialize_cpu();
        let result = cpu.int_to_binary(28);
        let expected = [0,0,0,1,1,1,0,0];
        assert_eq!(result, expected);
    }
}