use crate::assembler::{Instruction, InstructionCommand, InstructionRegister};
use std::convert::TryFrom;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub fn initialize_cpu() -> Cpu {
    Cpu {
        register: vec![0; 8],
        flags: vec![false; 8],
    }
}

#[derive(Debug)]
pub struct Cpu {
    register: Vec<i8>,

    // S Z x A x P x C
    flags: Vec<bool>,
}

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

impl Cpu {
    fn get_register(&self, index: usize) -> i8 {
        self.register[index]
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
            InstructionCommand::Mvi => {
                self.execute_mvi(&instruction.registers[0], &instruction.intermediate)
            }
            InstructionCommand::Adi => self.execute_adi(&instruction.intermediate),
            InstructionCommand::Aci => self.execute_aci(&instruction.intermediate),
            InstructionCommand::Sui => self.execute_sui(&instruction.intermediate),
            InstructionCommand::Mov => self.execute_mov(&instruction.registers),
            InstructionCommand::Add => self.execute_add(&instruction.registers[0]),
            InstructionCommand::Adc => self.execute_adc(&instruction.registers[0]),
            InstructionCommand::Sub => self.execute_sub(&instruction.registers[0]),
            InstructionCommand::Inr => self.execute_inr(&instruction.registers[0]),
            InstructionCommand::Dcr => self.execute_dcr(&instruction.registers[0]),
            InstructionCommand::Ana => self.execute_ana(&instruction.registers[0]),
            InstructionCommand::Stc => self.execute_stc(),
            InstructionCommand::Cmc => self.execute_cmc(),
            InstructionCommand::Cma => self.execute_cma(),
            InstructionCommand::Rlc => self.execute_rlc(),
            InstructionCommand::Rrc => self.execute_rrc(),
            InstructionCommand::Ral => self.execute_ral(),
            InstructionCommand::Rar => self.execute_rar(),
            InstructionCommand::Ora => self.execute_ora(&instruction.registers[0]),
            InstructionCommand::Daa => self.execute_daa(),
            InstructionCommand::Hlt => self.execute_hlt(),
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
        let new_a = current_a.wrapping_add(source_value);

        if self.get_register(0) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(0) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }

        // if onecomplement representation added > 255 -> carry exists
        // example: 127 + 127
        // "x as u8 as u16" converts to onecomplement representation
        if (source_value as u8 as u16) + (current_a as u8 as u16) > 255 {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        self.change_register(0, new_a);
    }

    fn execute_aci(&mut self, intermediate: &[u8]) {
        let mut x = vec![0; 8];
        x[0..].clone_from_slice(intermediate);

        let source_value = self.binary_to_int(&mut x);
        let current_a = self.get_register(0);
        let new_a = current_a
            .wrapping_add(source_value)
            .wrapping_add(self.get_flag(Flag::C) as i8);

        if self.get_register(0) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(0) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }

        // if onecomplement representation added > 255 -> carry exists
        // example: 127 + 127
        // "x as u8 as u16" converts to onecomplement representation
        if (source_value as u8 as u16) + (current_a as u8 as u16) + (self.get_flag(Flag::C) as u16)
            > 255
        {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        self.change_register(0, new_a);
    }

    fn execute_sui(&mut self, intermediate: &[u8]) {
        let mut x = vec![0; 8];
        x[0..].clone_from_slice(intermediate);
        let source_value = self.binary_to_int(&mut x);
        let current_a = self.get_register(0);
        let new_a = current_a.wrapping_sub(source_value);

        self.change_register(0, new_a);

        if self.get_register(0) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(0) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }

        // if onecomplement representation subtraction < 0 -> set carry
        // "x as u8 as u16" converts to onecomplement representation
        if (current_a as u8 as u16).checked_sub(source_value as u8 as u16) != None {
            self.set_flag(Flag::C, false);
        } else {
            self.set_flag(Flag::C, true);
        }
    }

    fn execute_mov(&mut self, args: &[InstructionRegister]) {
        let source_value = self.get_register(args[0].to_index().into());

        let destination_index = args[1].to_index().into();
        self.change_register(destination_index, source_value);
    }

    fn execute_add(&mut self, arg: &InstructionRegister) {
        let source_value = self.get_register(arg.to_index().into());
        let current_a = self.get_register(0);
        let new_a = current_a.wrapping_add(source_value);

        self.change_register(0, new_a);

        if self.get_register(0) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(0) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }

        // if onecomplement representation added > 255 -> carry exists
        // example: 127 + 127
        // "x as u8 as u16" converts to onecomplement representation
        if (source_value as u8 as u16) + (current_a as u8 as u16) > 255 {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        self.change_register(0, new_a);
    }

    fn execute_adc(&mut self, arg: &InstructionRegister) {
        let source_value = self.get_register(arg.to_index().into());
        let current_a = self.get_register(0);

        let new_a = current_a + source_value + self.get_flag(Flag::C) as i8;

        if self.get_register(0) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(0) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }

        // if onecomplement representation added > 255 -> carry exists
        // example: 127 + 127
        // "x as u8 as u16" converts to onecomplement representation
        if (source_value as u8 as u16) + (current_a as u8 as u16) + self.get_flag(Flag::C) as u16
            > 255
        {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        self.change_register(0, new_a);
    }

    fn execute_sub(&mut self, args: &InstructionRegister) {
        let source_value = self.get_register(args.to_index().into());
        let current_a = self.get_register(0);
        let new_a = current_a.wrapping_sub(source_value);

        self.change_register(0, new_a);

        if self.get_register(0) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(0) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }

        // if onecomplement representation subtraction < 0 -> set carry
        // "x as u8 as u16" converts to onecomplement representation
        if (current_a as u8 as u16).checked_sub(source_value as u8 as u16) != None {
            self.set_flag(Flag::C, false);
        } else {
            self.set_flag(Flag::C, true);
        }
    }

    fn execute_inr(&mut self, arg: &InstructionRegister) {
        let new_value = self.get_register(arg.to_index().into()) + 1;

        self.change_register(arg.to_index().into(), new_value);

        if self.get_register(0) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(0) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }
    }

    fn execute_dcr(&mut self, arg: &InstructionRegister) {
        let new_value = self.get_register(arg.to_index().into()) - 1;

        self.change_register(arg.to_index().into(), new_value);

        if self.get_register(0) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(0) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }
    }

    fn execute_ana(&mut self, arg: &InstructionRegister) {
        let mut binary_a = self.int_to_binary(self.get_register(0));
        let binary_reg = self.int_to_binary(self.get_register(arg.to_index().into()));

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
                value += (digit * u8::pow(2, u32::try_from(index).unwrap())) as i8;
            }
        }
        self.change_register(0, value as i8);
    }

    fn set_flag(&mut self, flag: Flag, value: bool) {
        self.flags[flag.get_index()] = value;
    }

    fn get_flag(&self, flag: Flag) -> bool {
        self.flags[flag.get_index()]
    }

    fn execute_hlt(&mut self) {
        println!("Execution finished");
        println!("Final status: ");
        self.print_status();
        std::process::exit(0);
    }

    fn execute_stc(&mut self) {
        self.set_flag(Flag::C, true);
    }

    fn execute_cmc(&mut self) {
        self.set_flag(Flag::C, !self.get_flag(Flag::C));
    }

    fn execute_cma(&mut self) {
        // complement of twos-complement is always
        // -(num+1)

        self.change_register(0, !self.get_register(0));
    }

    // last bit can never be 1 after shift
    // that's why the case of carry=0 and last bit 1 can be ignored
    fn execute_rlc(&mut self) {
        let mut acc = self.get_register(0);
        if (acc >> 7) & 1 == 1 {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        acc <<= 1;

        if self.get_flag(Flag::C) {
            acc |= 1;
        }

        self.change_register(0, acc);
    }

    fn execute_rrc(&mut self) {
        let mut acc = self.get_register(0);
        if (acc << 7) & -128 == -128 {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        // convert to u8 to make sure LSR is used
        // otherwise most significant bit is 1 after shift
        acc = ((acc as u8) >> 1) as i8;

        if self.get_flag(Flag::C) {
            acc |= -128;
        }

        self.change_register(0, acc);
    }

    fn execute_ral(&mut self) {
        let mut acc = self.get_register(0);
        let flag = self.get_flag(Flag::C);
        if (acc >> 7) & 1 == 1 {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        acc <<= 1;

        if flag {
            acc |= 1;
        }

        self.change_register(0, acc);
    }

    fn execute_rar(&mut self) {
        let mut acc = self.get_register(0);
        let flag = self.get_flag(Flag::C);
        if (acc << 7) & -128 == -128 {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        // convert to u8 to make sure LSR is used
        // otherwise most significant bit is 1 after shift
        acc = ((acc as u8) >> 1) as i8;

        if flag {
            acc |= -128;
        }

        self.change_register(0, acc);
    }

    fn execute_ora(&mut self, arg: &InstructionRegister) {
        let mut acc = self.get_register(0);
        acc |= self.get_register(arg.to_index().into());

        self.change_register(0, acc);
        self.set_flag(Flag::C, false);
    }

    fn execute_daa(&mut self) {
        let mut acc = self.get_register(0);

        // check if 4 least significant bits are > 9
        if (acc & 15) > 9 || self.get_flag(Flag::A) {
            acc = acc.wrapping_add(6);

            // check if carry out happens
            if (self.get_register(0) & -16) != (acc & -16) {
                self.set_flag(Flag::A, true);
            } else {
                self.set_flag(Flag::A, false);
            }
        }

        // check if 4 most significant bits are > 9
        // increment 4 most significant bits by 6
        // since its 4 most significant, +6 = +96
        let most_significant_bits = (((acc & -16) as u8) >> 4) as i8;
        if most_significant_bits > 9 || self.get_flag(Flag::C) {
            // if onecomplement representation added > 255 -> carry exists
            // example: 127 + 127
            // "x as u8 as u16" converts to onecomplement representation
            if (acc as u8 as u16) + (96u16) > 255 {
                self.set_flag(Flag::C, true);
            } else {
                self.set_flag(Flag::C, false);
            }
            acc = acc.wrapping_add(96);
        }

        self.change_register(0, acc);
    }

    fn binary_to_int(&self, intermediate: &mut [u8]) -> i8 {
        if intermediate[0] == 1 {
            // subtract 1 from intermediate
            let mut index = intermediate.len() - 1;
            while index > 0 {
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
                value += digit * u8::pow(2, u32::try_from(index).unwrap());
            }
            -(value as i8)
        } else {
            let mut value = 0;
            for (index, digit) in intermediate.iter().rev().enumerate() {
                value += digit * u8::pow(2, u32::try_from(index).unwrap());
            }
            value as i8
        }
    }

    fn int_to_binary(&self, value: i8) -> Vec<u8> {
        let binary_string = format!("{:08b}", value);

        let mut result = Vec::new();
        for c in binary_string.chars() {
            result.push((c as u8) - 48);
        }
        result
    }

    fn print_status(&self) {
        for i in 0..7 {
            println!(
                "{}: {:#010b} ({})",
                i,
                self.get_register(i),
                self.get_register(i)
            );
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

#[cfg(test)]
mod tests {
    use super::initialize_cpu;
    use crate::cpu::{Flag, InstructionRegister};

    #[test]
    fn test_execute_mvi() {
        let mut cpu = initialize_cpu();

        cpu.execute_mvi(&InstructionRegister::A, &[0, 0, 0, 0, 1, 1, 1, 0]);
        assert_eq!(cpu.get_register(0), 14);
    }

    #[test]
    fn test_execute_mov() {
        let mut cpu = initialize_cpu();
        cpu.change_register(0, 10);

        cpu.execute_mov(&[InstructionRegister::A, InstructionRegister::B]);
        assert_eq!(cpu.get_register(1), 10);
    }

    #[test]
    fn test_execute_add() {
        let mut cpu = initialize_cpu();
        cpu.change_register(0, 5);
        cpu.set_flag(Flag::Z, true);

        cpu.execute_add(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), 10);
        assert_eq!(cpu.get_flag(Flag::Z), false);

        cpu.change_register(0, -5);
        cpu.execute_add(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), -10);
        assert_eq!(cpu.get_flag(Flag::S), true);

        cpu.change_register(0, 127);
        cpu.change_register(1, 127);
        cpu.set_flag(Flag::C, true);
        cpu.execute_add(&InstructionRegister::B);
        assert_eq!(cpu.get_register(0), -2);
        assert_eq!(cpu.get_flag(Flag::C), false);

        cpu.change_register(0, -64);
        cpu.change_register(1, 64);
        cpu.execute_add(&InstructionRegister::B);
        assert_eq!(cpu.get_register(0), 0);
        assert_eq!(cpu.get_flag(Flag::C), true);
    }

    #[test]
    fn test_execute_adc() {
        let mut cpu = initialize_cpu();
        cpu.change_register(0, 5);
        cpu.set_flag(Flag::Z, true);

        cpu.change_register(0, 10);
        cpu.set_flag(Flag::C, false);
        cpu.execute_adc(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), 20);

        cpu.change_register(0, 10);
        cpu.set_flag(Flag::C, true);
        cpu.execute_adc(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), 21);

        cpu.change_register(0, -64);
        cpu.change_register(1, 63);
        cpu.set_flag(Flag::C, true);
        cpu.execute_adc(&InstructionRegister::B);
        assert_eq!(cpu.get_register(0), 0);
        assert_eq!(cpu.get_flag(Flag::C), true);

        cpu.change_register(0, 15);
        cpu.change_register(1, 63);
        cpu.set_flag(Flag::C, true);
        cpu.execute_adc(&InstructionRegister::B);
        assert_eq!(cpu.get_register(0), 79);
        assert_eq!(cpu.get_flag(Flag::C), false);
    }

    #[test]
    fn test_execute_adi() {
        let mut cpu = initialize_cpu();
        cpu.change_register(0, 5);
        cpu.set_flag(Flag::Z, true);

        cpu.execute_adi(&cpu.int_to_binary(5));
        assert_eq!(cpu.get_register(0), 10);
        assert_eq!(cpu.get_flag(Flag::Z), false);

        cpu.change_register(0, -5);
        cpu.execute_adi(&cpu.int_to_binary(-5));
        assert_eq!(cpu.get_register(0), -10);
        assert_eq!(cpu.get_flag(Flag::S), true);

        cpu.change_register(0, -64);
        cpu.set_flag(Flag::C, true);
        cpu.execute_adi(&cpu.int_to_binary(64));
        assert_eq!(cpu.get_register(0), 0);
        assert_eq!(cpu.get_flag(Flag::C), true);

        cpu.change_register(0, 127);
        cpu.execute_adi(&cpu.int_to_binary(127));
        assert_eq!(cpu.get_register(0), -2);
        assert_eq!(cpu.get_flag(Flag::C), false);
    }

    #[test]
    fn test_execute_aci() {
        let mut cpu = initialize_cpu();
        cpu.change_register(0, 5);
        cpu.set_flag(Flag::Z, true);

        cpu.execute_aci(&cpu.int_to_binary(5));
        assert_eq!(cpu.get_register(0), 10);
        assert_eq!(cpu.get_flag(Flag::Z), false);

        cpu.change_register(0, -5);
        cpu.execute_aci(&cpu.int_to_binary(-5));
        assert_eq!(cpu.get_register(0), -10);
        assert_eq!(cpu.get_flag(Flag::S), true);

        cpu.change_register(0, -64);
        cpu.set_flag(Flag::C, true);
        cpu.execute_aci(&cpu.int_to_binary(64));
        assert_eq!(cpu.get_register(0), 1);
        assert_eq!(cpu.get_flag(Flag::C), true);

        cpu.change_register(0, 127);
        cpu.set_flag(Flag::C, false);
        cpu.execute_aci(&cpu.int_to_binary(127));
        assert_eq!(cpu.get_register(0), -2);
        assert_eq!(cpu.get_flag(Flag::C), false);

        cpu.set_flag(Flag::C, true);
        cpu.change_register(0, 0);
        cpu.execute_aci(&cpu.int_to_binary(4));
        assert_eq!(cpu.get_register(0), 5);

        cpu.set_flag(Flag::C, false);
        cpu.change_register(0, 0);
        cpu.execute_aci(&cpu.int_to_binary(4));
        assert_eq!(cpu.get_register(0), 4);
    }

    #[test]
    fn test_excute_sui() {
        let mut cpu = initialize_cpu();

        cpu.set_flag(Flag::Z, false);
        cpu.change_register(0, 5);
        cpu.execute_sui(&cpu.int_to_binary(5));
        assert_eq!(cpu.get_register(0), 0);
        assert_eq!(cpu.get_flag(Flag::Z), true);

        cpu.set_flag(Flag::Z, true);
        cpu.change_register(0, -5);
        cpu.execute_sui(&cpu.int_to_binary(8));
        assert_eq!(cpu.get_register(0), -13);
        assert_eq!(cpu.get_flag(Flag::Z), false);

        cpu.set_flag(Flag::S, false);
        cpu.change_register(0, 10);
        cpu.execute_sui(&cpu.int_to_binary(16));
        assert_eq!(cpu.get_register(0), -6);
        assert_eq!(cpu.get_flag(Flag::S), true);

        cpu.set_flag(Flag::S, true);
        cpu.change_register(0, 10);
        cpu.execute_sui(&cpu.int_to_binary(1));
        assert_eq!(cpu.get_register(0), 9);
        assert_eq!(cpu.get_flag(Flag::S), false);

        cpu.change_register(0, 127);
        cpu.execute_sui(&cpu.int_to_binary(-1));
        assert_eq!(cpu.get_register(0), -128);
        assert_eq!(cpu.get_flag(Flag::C), true);

        cpu.set_flag(Flag::C, true);
        cpu.change_register(0, 10);
        cpu.execute_sui(&cpu.int_to_binary(1));
        assert_eq!(cpu.get_register(0), 9);
        assert_eq!(cpu.get_flag(Flag::C), false);
    }

    #[test]
    fn test_execute_sub() {
        let mut cpu = initialize_cpu();
        cpu.change_register(0, 5);

        cpu.execute_sub(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), 0);
        assert_eq!(cpu.get_flag(Flag::Z), true);

        cpu.change_register(0, -5);
        cpu.execute_sub(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), 0);

        cpu.change_register(0, 127);
        cpu.change_register(1, -1);
        cpu.execute_sub(&InstructionRegister::B);
        assert_eq!(cpu.get_register(0), -128);
        assert_eq!(cpu.get_flag(Flag::C), true);

        cpu.change_register(0, -59);
        cpu.change_register(1, -98);
        cpu.set_flag(Flag::C, true);
        cpu.execute_sub(&InstructionRegister::B);
        assert_eq!(cpu.get_register(0), 39);
        assert_eq!(cpu.get_flag(Flag::C), false);

        cpu.change_register(0, 12);
        cpu.change_register(1, -15);
        cpu.set_flag(Flag::C, false);
        cpu.execute_sub(&InstructionRegister::B);
        assert_eq!(cpu.get_register(0), 27);
        assert_eq!(cpu.get_flag(Flag::C), true);
    }

    #[test]
    fn test_execute_inr() {
        let mut cpu = initialize_cpu();

        cpu.execute_inr(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), 1);

        cpu.change_register(0, -2);
        cpu.execute_inr(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), -1);
        assert_eq!(cpu.get_flag(Flag::S), true);
    }

    #[test]
    fn test_execute_dcr() {
        let mut cpu = initialize_cpu();
        cpu.change_register(0, 1);

        cpu.execute_dcr(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), 0);
        assert_eq!(cpu.get_flag(Flag::Z), true);

        cpu.change_register(0, -1);

        cpu.execute_dcr(&InstructionRegister::A);
        assert_eq!(cpu.get_register(0), -2);
        assert_eq!(cpu.get_flag(Flag::S), true);
    }

    #[test]
    fn test_execute_ana() {
        let mut cpu = initialize_cpu();
        cpu.change_register(0, -10);
        cpu.change_register(1, -10);

        cpu.execute_ana(&InstructionRegister::B);
        assert_eq!(cpu.get_register(0), -10);

        // -15 11110001
        // -10 11110110
        // ANA 11110000

        cpu.change_register(0, -15);
        cpu.execute_ana(&InstructionRegister::B);
        assert_eq!(cpu.get_register(0), -16);
    }

    #[test]
    fn test_execute_stc() {
        let mut cpu = initialize_cpu();

        cpu.execute_stc();
        assert_eq!(cpu.get_flag(Flag::C), true);
    }

    #[test]
    fn test_execute_cmc() {
        let mut cpu = initialize_cpu();

        cpu.execute_cmc();
        assert_eq!(cpu.get_flag(Flag::C), true);

        cpu.execute_cmc();
        assert_eq!(cpu.get_flag(Flag::C), false);
    }

    #[test]
    fn test_execute_cma() {
        let mut cpu = initialize_cpu();

        cpu.change_register(0, 74);
        cpu.execute_cma();
        assert_eq!(cpu.get_register(0), -75);

        cpu.change_register(0, -45);
        cpu.execute_cma();
        assert_eq!(cpu.get_register(0), 44);
    }

    #[test]
    fn test_execute_rlc() {
        let mut cpu = initialize_cpu();

        // negative with carry
        cpu.set_flag(Flag::C, false);
        cpu.change_register(0, -14);
        cpu.execute_rlc();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(0), -27);

        // negative without carry
        cpu.set_flag(Flag::C, false);
        cpu.change_register(0, -128);
        cpu.execute_rlc();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(0), 1);

        // positive
        cpu.set_flag(Flag::C, true);
        cpu.change_register(0, 24);
        cpu.execute_rlc();
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(0), 48);
    }

    #[test]
    fn test_execute_rrc() {
        let mut cpu = initialize_cpu();

        // negative without carry
        cpu.set_flag(Flag::C, true);
        cpu.change_register(0, -14);
        cpu.execute_rrc();
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(0), 121);

        // negative with carry
        cpu.set_flag(Flag::C, false);
        cpu.change_register(0, -13);
        cpu.execute_rrc();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(0), -7);

        // positive without carry
        cpu.set_flag(Flag::C, true);
        cpu.change_register(0, 12);
        cpu.execute_rrc();
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(0), 6);

        // positive with carry
        cpu.set_flag(Flag::C, false);
        cpu.change_register(0, 13);
        cpu.execute_rrc();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(0), -122);
    }

    #[test]
    fn test_execute_ral() {
        let mut cpu = initialize_cpu();

        // false -> true
        cpu.set_flag(Flag::C, false);
        cpu.change_register(0, -75);
        cpu.execute_ral();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(0), 106);

        // true -> true
        cpu.set_flag(Flag::C, true);
        cpu.change_register(0, -75);
        cpu.execute_ral();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(0), 107);

        // false -> false
        cpu.set_flag(Flag::C, false);
        cpu.change_register(0, 12);
        cpu.execute_ral();
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(0), 24);

        // true -> false
        cpu.set_flag(Flag::C, true);
        cpu.change_register(0, 12);
        cpu.execute_ral();
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(0), 25);
    }

    #[test]
    fn test_execute_rar() {
        let mut cpu = initialize_cpu();

        // true -> false
        cpu.set_flag(Flag::C, true);
        cpu.change_register(0, 106);
        cpu.execute_rar();
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(0), -75);

        // false -> false
        cpu.set_flag(Flag::C, false);
        cpu.change_register(0, 106);
        cpu.execute_rar();
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(0), 53);

        // false -> true
        cpu.set_flag(Flag::C, false);
        cpu.change_register(0, 53);
        cpu.execute_rar();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(0), 26);

        // true -> true
        cpu.set_flag(Flag::C, true);
        cpu.change_register(0, 53);
        cpu.execute_rar();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(0), -102);
    }

    #[test]
    fn test_execute_ora() {
        let mut cpu = initialize_cpu();

        cpu.set_flag(Flag::C, true);
        cpu.change_register(0, 51);
        cpu.change_register(1, 15);
        cpu.execute_ora(&InstructionRegister::B);
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(0), 63);

        cpu.set_flag(Flag::C, false);
        cpu.change_register(0, -1);
        cpu.change_register(1, 0);
        cpu.execute_ora(&InstructionRegister::B);
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(0), -1);
    }

    #[test]
    fn test_execute_daa() {
        let mut cpu = initialize_cpu();

        // neither carry bit are set
        cpu.set_flag(Flag::A, true);
        cpu.change_register(0, -101);
        cpu.execute_daa();
        assert_eq!(cpu.get_register(0), 1);
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_flag(Flag::A), true);
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

        let mut intermediate = [1, 1, 1, 1, 0, 0, 0, 1];
        let result = cpu.binary_to_int(&mut intermediate);
        assert_eq!(result, -15);

        let mut intermediate = [0, 1, 1, 1, 0, 0, 0, 1];
        let result = cpu.binary_to_int(&mut intermediate);
        assert_eq!(result, 113);
    }

    #[test]
    fn test_int_to_binary() {
        let cpu = initialize_cpu();
        let result = cpu.int_to_binary(28);
        let expected = [0, 0, 0, 1, 1, 1, 0, 0];
        assert_eq!(result, expected);
    }
}
