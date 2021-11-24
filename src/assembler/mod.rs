pub use crate::assembler::parser::{
    Instruction, InstructionArgument, InstructionCommand, InstructionRegister,
    InstructionRegisterPair,
};
use std::fs::File;
use std::io::{Read, Write};

mod parser;

#[derive(Debug)]
pub struct Assembler {
    input_asm: String,
    output_bin: String,
}

impl Assembler {
    pub fn new(input_asm: String, output_bin: String) -> Assembler {
        Assembler {
            input_asm,
            output_bin,
        }
    }

    pub fn assemble(&self) {
        let parse_result = parser::parse(self.input_asm.to_owned());
        let instructions = parse_result.0;

        // write to file
        // TODO maybe write hex data instead of binary
        let mut file = File::create(&self.output_bin).unwrap();
        for instruction in instructions {
            let encoding = &instruction.encode();
            file.write_all(encoding).unwrap();
        }
    }

    pub fn disassemble(&self, input_bin: String) -> Vec<Instruction> {
        let mut file = File::open(input_bin).unwrap();
        let mut binary_data = Vec::new();

        file.read_to_end(&mut binary_data).unwrap();

        if binary_data.len() % 8 != 0 {
            panic!("Data is not proper length!");
        }

        let mut raw_instructions = Vec::new();
        for chunk in binary_data.chunks(8) {
            raw_instructions.push(chunk);
        }

        self.parse_binary_instructions(&raw_instructions)
    }

    fn parse_binary_instructions(&self, raw_instructions: &[&[u8]]) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        let mut index = 0;
        while index < raw_instructions.len() {
            // pretty ugly, maybe there is a better solution with match or something

            let instruction: Instruction;
            // instructions that take up more than one byte (intermediates)
            // MVI
            if raw_instructions[index][0..2] == [0, 0] && raw_instructions[index][5..] == [1, 1, 0]
            {
                let register = InstructionRegister::decode(&raw_instructions[index][2..5]);
                let intermediate = parser::binary_to_int(&mut raw_instructions[index + 1].to_vec());
                instruction = Instruction::IntermediateRegister(
                    InstructionCommand::Mvi,
                    intermediate,
                    register,
                );
            // ADI
            } else if raw_instructions[index] == [1, 1, 0, 0, 0, 1, 1, 0] {
                let intermediate = parser::binary_to_int(&mut raw_instructions[index + 1].to_vec());
                instruction = Instruction::Intermediate(InstructionCommand::Adi, intermediate);
            // ACI
            } else if raw_instructions[index] == [1, 1, 0, 0, 1, 1, 1, 0] {
                let intermediate = parser::binary_to_int(&mut raw_instructions[index + 1].to_vec());
                instruction = Instruction::Intermediate(InstructionCommand::Aci, intermediate);
            // SUI
            } else if raw_instructions[index] == [1, 1, 0, 1, 0, 1, 1, 0] {
                let intermediate = parser::binary_to_int(&mut raw_instructions[index + 1].to_vec());
                instruction = Instruction::Intermediate(InstructionCommand::Sui, intermediate);

            // instructions without registers
            // HLT
            } else if raw_instructions[index] == [0, 1, 1, 1, 0, 1, 1, 0] {
                instruction = Instruction::NoRegister(InstructionCommand::Hlt);

            // STC
            } else if raw_instructions[index] == [0, 0, 1, 1, 0, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Stc);

            // CMC
            } else if raw_instructions[index] == [0, 0, 1, 1, 1, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Cmc);

            // CMA
            } else if raw_instructions[index] == [0, 0, 1, 0, 1, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Cma);

            // RLC
            } else if raw_instructions[index] == [0, 0, 0, 0, 0, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Rlc);

            // RRC
            } else if raw_instructions[index] == [0, 0, 0, 0, 1, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Rrc);

            // RAL
            } else if raw_instructions[index] == [0, 0, 0, 1, 0, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Ral);

            // RAR
            } else if raw_instructions[index] == [0, 0, 0, 1, 1, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Rar);

            // DAA
            } else if raw_instructions[index] == [0, 0, 1, 0, 0, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Daa);

            // XCHG
            } else if raw_instructions[index] == [1, 1, 1, 0, 1, 0, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Xchg);

            // SPHL
            } else if raw_instructions[index] == [1, 1, 1, 1, 1, 0, 0, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Sphl);

            // XTHL
            } else if raw_instructions[index] == [1, 1, 1, 0, 0, 0, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Xthl);

            // instructions with 1 argument in the end
            // ADD
            } else if raw_instructions[index][0..5] == [1, 0, 0, 0, 0] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Add, register);

            // ADC
            } else if raw_instructions[index][0..5] == [1, 0, 0, 0, 1] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Adc, register);

            // SUB
            } else if raw_instructions[index][0..5] == [1, 0, 0, 1, 0] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Sub, register);

            // ANA
            } else if raw_instructions[index][0..5] == [1, 0, 1, 0, 0] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Ana, register);

            // ORA
            } else if raw_instructions[index][0..5] == [1, 0, 1, 1, 0] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Ora, register);

            // CMP
            } else if raw_instructions[index][0..5] == [1, 0, 1, 1, 1] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Cmp, register);

            // XRA
            } else if raw_instructions[index][0..5] == [1, 0, 1, 0, 1] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Xra, register);

            // SBB
            } else if raw_instructions[index][0..5] == [1, 0, 0, 1, 1] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Sbb, register);

            // instructions with 1 argument in the middle
            // instructions with a register pair
            // STAX
            } else if raw_instructions[index][0..3] == [0, 0, 0]
                && raw_instructions[index][4..] == [0, 0, 1, 0]
            {
                let register_pair: InstructionRegisterPair;
                if raw_instructions[index][4] == 0 {
                    register_pair = InstructionRegisterPair::BC;
                } else {
                    register_pair = InstructionRegisterPair::DE;
                }

                instruction = Instruction::PairRegister(InstructionCommand::Stax, register_pair);

            // LDAX
            } else if raw_instructions[index][0..3] == [0, 0, 0]
                && raw_instructions[index][4..] == [1, 0, 1, 0]
            {
                let register_pair: InstructionRegisterPair;
                if raw_instructions[index][4] == 0 {
                    register_pair = InstructionRegisterPair::BC;
                } else {
                    register_pair = InstructionRegisterPair::DE;
                }

                instruction = Instruction::PairRegister(InstructionCommand::Ldax, register_pair);

            // instructions with 1 register in the middle
            // INR
            } else if raw_instructions[index][0..2] == [0, 0]
                && raw_instructions[index][5..] == [1, 0, 0]
            {
                let register = InstructionRegister::decode(&raw_instructions[index][2..5]);
                instruction = Instruction::SingleRegister(InstructionCommand::Inr, register);
            // DCR
            } else if raw_instructions[index][0..2] == [0, 0]
                && raw_instructions[index][5..] == [1, 0, 1]
            {
                let register = InstructionRegister::decode(&raw_instructions[index][2..5]);
                instruction = Instruction::SingleRegister(InstructionCommand::Dcr, register);

            // instructions with 2 registers
            // MOV
            } else if raw_instructions[index][0..2] == [0, 1] {
                let registers = (
                    InstructionRegister::decode(&raw_instructions[index][2..5]),
                    InstructionRegister::decode(&raw_instructions[index][5..]),
                );

                instruction = Instruction::DoubleRegister(InstructionCommand::Mov, registers);
            } else {
                panic!("Invalid instruction!");
            }

            // skip next byte since its the intermediate of the instruction that was just parsed
            if matches!(instruction, Instruction::Intermediate(_, _))
                || matches!(instruction, Instruction::IntermediateRegister(_, _, _))
            {
                index += 2;
            } else {
                index += 1;
            }
            instructions.push(instruction);
        }
        instructions
    }
}

#[cfg(test)]
mod tests {
    use super::Assembler;
    use crate::assembler::parser::{Instruction, InstructionCommand, InstructionRegister};
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_new() {
        let assembler = Assembler::new("test.asm".to_owned(), "output".to_owned());
        assert_eq!("test.asm", assembler.input_asm);
        assert_eq!("output", assembler.output_bin);
    }

    #[test]
    fn test_assemble() {
        let assembler = Assembler::new("test.asm".to_owned(), "output".to_owned());
        assembler.assemble();

        let mut file = File::open("output").unwrap();
        let mut binary_data = Vec::new();

        file.read_to_end(&mut binary_data).unwrap();

        assert_eq!(binary_data.len() % 8, 0);

        let mut bytes = binary_data.chunks(8);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 1, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 1, 1, 1, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 1, 1, 1, 1, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 1, 0, 0, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 0, 0, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 1, 0, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 1, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 1, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 0, 0, 1, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 1, 1, 0, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 0, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 0, 1, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 0, 1, 0, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 0, 1, 1, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 1, 0, 1, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 1, 0, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 1, 1, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 1, 1, 0, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 0, 0, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 1, 1, 0, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 1, 1, 1, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 1, 0, 1, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 1, 1, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 0, 1, 0, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 1, 1, 0, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 0, 0, 0, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 1, 1, 1, 0, 1, 1, 0]);
    }

    // TODO rework this test
    #[test]
    fn test_disassemble() {
        let assembler = Assembler::new("test.asm".to_owned(), "output".to_owned());
        assembler.assemble();

        let instructions = assembler.disassemble("output".to_owned());

        for (i, instruction) in instructions.iter().enumerate() {
            match instruction {
                Instruction::NoRegister(cmd) => match cmd {
                    InstructionCommand::Stc => {
                        assert_eq!(8, i);
                    }
                    InstructionCommand::Cmc => {
                        assert_eq!(9, i);
                    }
                    InstructionCommand::Cma => {
                        assert_eq!(10, i);
                    }
                    InstructionCommand::Rlc => {
                        assert_eq!(14, i);
                    }
                    InstructionCommand::Rrc => {
                        assert_eq!(15, i);
                    }
                    InstructionCommand::Ral => {
                        assert_eq!(16, i);
                    }
                    InstructionCommand::Rar => {
                        assert_eq!(17, i);
                    }
                    InstructionCommand::Daa => {
                        assert_eq!(19, i);
                    }
                    InstructionCommand::Xchg => {
                        assert_eq!(25, i);
                    }
                    InstructionCommand::Sphl => {
                        assert_eq!(26, i);
                    }
                    InstructionCommand::Xthl => {
                        assert_eq!(27, i);
                    }
                    InstructionCommand::Hlt => {
                        assert_eq!(28, i);
                    }
                    _ => panic!("invalid instruction"),
                },
                Instruction::SingleRegister(cmd, register) => match cmd {
                    InstructionCommand::Ana => {
                        assert_eq!(2, i);
                        assert!(matches!(register, InstructionRegister::B))
                    }
                    InstructionCommand::Add => {
                        assert_eq!(3, i);
                        assert!(matches!(register, InstructionRegister::A))
                    }
                    InstructionCommand::Sub => {
                        assert_eq!(4, i);
                        assert!(matches!(register, InstructionRegister::A))
                    }
                    InstructionCommand::Inr => {
                        assert_eq!(5, i);
                        assert!(matches!(register, InstructionRegister::A))
                    }
                    InstructionCommand::Dcr => {
                        assert_eq!(6, i);
                        assert!(matches!(register, InstructionRegister::A))
                    }
                    InstructionCommand::Adc => {
                        assert_eq!(11, i);
                        assert!(matches!(register, InstructionRegister::C))
                    }
                    InstructionCommand::Ora => {
                        assert_eq!(18, i);
                        assert!(matches!(register, InstructionRegister::B))
                    }
                    InstructionCommand::Cmp => {
                        assert_eq!(22, i);
                        assert!(matches!(register, InstructionRegister::B))
                    }
                    InstructionCommand::Xra => {
                        assert_eq!(23, i);
                        assert!(matches!(register, InstructionRegister::B))
                    }
                    InstructionCommand::Sbb => {
                        assert_eq!(24, i);
                        assert!(matches!(register, InstructionRegister::B));
                    }
                    _ => panic!("invalid instruction"),
                },
                Instruction::DoubleRegister(cmd, registers) => match cmd {
                    InstructionCommand::Mov => {
                        assert_eq!(1, i);
                        assert!(matches!(registers.0, InstructionRegister::A));
                        assert!(matches!(registers.1, InstructionRegister::B));
                    }
                    _ => panic!("invalid instruction"),
                },
                Instruction::Intermediate(cmd, intermediate) => match cmd {
                    InstructionCommand::Adi => {
                        assert_eq!(7, i);
                        assert_eq!(-103, *intermediate);
                    }
                    InstructionCommand::Aci => {
                        assert_eq!(12, i);
                        assert_eq!(12, *intermediate);
                    }
                    InstructionCommand::Sui => {
                        assert_eq!(13, i);
                        assert_eq!(12, *intermediate);
                    }
                    _ => panic!("invalid instruction"),
                },
                Instruction::IntermediateRegister(cmd, intermediate, register) => match cmd {
                    InstructionCommand::Mvi => {
                        assert_eq!(0, i);
                        assert!(matches!(register, InstructionRegister::A));
                        assert_eq!(28, *intermediate);
                    }
                    _ => panic!("invalid instruction"),
                },
                Instruction::PairRegister(cmd, register_pair) => match cmd {
                    InstructionCommand::Stax => {
                        assert_eq!(20, i);
                        let registers = register_pair.get_registers();
                        assert!(matches!(registers.0, InstructionRegister::B));
                        assert!(matches!(registers.1, InstructionRegister::C));
                    }
                    InstructionCommand::Ldax => {
                        assert_eq!(21, i);
                        let registers = register_pair.get_registers();
                        assert!(matches!(registers.0, InstructionRegister::D));
                        assert!(matches!(registers.1, InstructionRegister::E));
                    }
                    _ => panic!("invalid instruction"),
                },
            }
        }
    }
}
