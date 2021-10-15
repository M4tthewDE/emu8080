use std::fs::File;
use std::io::{Write, Read};
use crate::assembler::parser::Encoding;
pub use crate::assembler::parser::{InstructionRegister, Instruction, InstructionCommand, InstructionType};

mod parser;

#[derive(Debug)]
pub struct Assembler{
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
        let instructions = parser::parse(self.input_asm.to_owned());

        // write to file
        // TODO maybe write hex data instead of binary
        let mut file = File::create(&self.output_bin).unwrap();
        for instruction in instructions {
            let encoding = &instruction.encode();
            for byte in encoding {
                file.write_all(byte).unwrap();
            }
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
            if raw_instructions[index][0..2] == [0, 0]
                && !matches!(InstructionRegister::decode(&raw_instructions[index][2..5]), InstructionRegister::Invalid) &&
                raw_instructions[index][5..] == [1, 1, 0] {

                instruction = Instruction {
                    variant: InstructionType::IntermediateReg,
                    command: InstructionCommand::Mvi,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][2..5])],
                    intermediate: raw_instructions[index+1].to_vec(),
                };
            // ADI
            } else if raw_instructions[index] == [1,1,0,0,0,1,1,0] {
                instruction = Instruction {
                    variant: InstructionType::Intermediate,
                    command: InstructionCommand::Adi,
                    registers: vec![],
                    intermediate: raw_instructions[index+1].to_vec(),
                };
            // instructions without registers
            // HLT
            } else if raw_instructions[index] == [0, 1, 1, 1, 0, 1, 1, 0] {
                instruction = Instruction {
                    variant: InstructionType::NoReg,
                    command: InstructionCommand::Hlt,
                    registers: Vec::new(),
                    intermediate: Vec::new(),
                };
            // instructions with 1 argument in the end
            // ADD
            } else if raw_instructions[index][0..5] == [1, 0, 0, 0, 0] 
                && !matches!(InstructionRegister::decode(&raw_instructions[index][5..]), InstructionRegister::Invalid) {
                    
                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Add,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][5..])],
                    intermediate: Vec::new(),
                }
            // SUB
            } else if raw_instructions[index][0..5] == [1, 0, 0, 1, 0] 
                && !matches!(InstructionRegister::decode(&raw_instructions[index][5..]), InstructionRegister::Invalid) {
                    
                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Sub,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][5..])],
                    intermediate: Vec::new(),
                }
            // ANA 
            } else if raw_instructions[index][0..5] == [1, 0, 1, 0, 0] 
                && !matches!(InstructionRegister::decode(&raw_instructions[index][5..]), InstructionRegister::Invalid) {
                    
                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Ana,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][5..])],
                    intermediate: Vec::new(),
                }
            // instructions with 1 argument in the middle
            // INR
            } else if raw_instructions[index][0..2] == [0, 0] && raw_instructions[index][5..] == [1, 0, 0] 
                && !matches!(InstructionRegister::decode(&raw_instructions[index][2..5]), InstructionRegister::Invalid) {
                    
                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Inr,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][2..5])],
                    intermediate: Vec::new(),
                }
            // DCR
            } else if raw_instructions[index][0..2] == [0, 0] && raw_instructions[index][5..] == [1, 0, 1] 
                && !matches!(InstructionRegister::decode(&raw_instructions[index][2..5]), InstructionRegister::Invalid) {

                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Dcr,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][2..5])],
                    intermediate: Vec::new(),
                }
            // instructions with 2 registers
            // MOV
            } else if raw_instructions[index][0..2] == [0, 1]
                && !matches!(InstructionRegister::decode(&raw_instructions[index][2..5]), InstructionRegister::Invalid)
                && !matches!(InstructionRegister::decode(&raw_instructions[index][5..]), InstructionRegister::Invalid) {

                let args = vec![
                                InstructionRegister::decode(&raw_instructions[index][2..5]),
                                InstructionRegister::decode(&raw_instructions[index][5..]),
                            ];
                
                instruction = Instruction {
                    variant: InstructionType::DoubleReg,
                    command: InstructionCommand::Mov,
                    registers: args,
                    intermediate: Vec::new(),
                }
            } else {
                panic!("Invalid instruction!");
            }

            // skip next byte since its the intermediate of the instruction that was just parsed
            if matches!(instruction.variant, InstructionType::Intermediate) ||
                matches!(instruction.variant, InstructionType::IntermediateReg)
            {
                index +=2;
            } else {
                index +=1;
            }
            instructions.push(instruction);
        }
        instructions
    }
}

#[cfg(test)]
mod tests {
    use super::Assembler;
    use std::fs::File;
    use std::io::Read;
    use super::parser::{InstructionType, InstructionCommand, InstructionRegister};

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
        assert_eq!(bytes.next().unwrap(), [0,0,1,1,1,1,1,0]);
        assert_eq!(bytes.next().unwrap(), [0,0,0,1,1,1,0,0]);
        assert_eq!(bytes.next().unwrap(), [0,1,1,1,1,0,0,0]);
        assert_eq!(bytes.next().unwrap(), [1,0,1,0,0,0,0,0]);
        assert_eq!(bytes.next().unwrap(), [1,0,0,0,0,1,1,1]);
        assert_eq!(bytes.next().unwrap(), [1,0,0,1,0,1,1,1]);
        assert_eq!(bytes.next().unwrap(), [0,0,1,1,1,1,0,0]);
        assert_eq!(bytes.next().unwrap(), [0,0,1,1,1,1,0,1]);
        assert_eq!(bytes.next().unwrap(), [1,1,0,0,0,1,1,0]);
        assert_eq!(bytes.next().unwrap(), [1,0,0,1,1,0,0,1]);
        assert_eq!(bytes.next().unwrap(), [0,1,1,1,0,1,1,0]);
    }

    #[test]
    fn test_disassemble() {
        let assembler = Assembler::new("test.asm".to_owned(), "output".to_owned());
        assembler.assemble();

        let instructions = assembler.disassemble("output".to_owned());
        assert_eq!(instructions.len(), 9);

        assert!(matches!(instructions[0].variant, InstructionType::IntermediateReg));
        assert!(matches!(instructions[0].command, InstructionCommand::Mvi));
        assert!(matches!(instructions[0].registers[0], InstructionRegister::A));
        assert_eq!(instructions[0].intermediate, [0,0,0,1,1,1,0,0]);

        assert!(matches!(instructions[1].variant, InstructionType::DoubleReg));
        assert!(matches!(instructions[1].command, InstructionCommand::Mov));
        assert!(matches!(instructions[1].registers[0], InstructionRegister::A));
        assert!(matches!(instructions[1].registers[1], InstructionRegister::B));

        assert!(matches!(instructions[2].variant, InstructionType::SingleReg));
        assert!(matches!(instructions[2].command, InstructionCommand::Ana));
        assert!(matches!(instructions[2].registers[0], InstructionRegister::B));

        assert!(matches!(instructions[3].variant, InstructionType::SingleReg));
        assert!(matches!(instructions[3].command, InstructionCommand::Add));
        assert!(matches!(instructions[3].registers[0], InstructionRegister::A));

        assert!(matches!(instructions[4].variant, InstructionType::SingleReg));
        assert!(matches!(instructions[4].command, InstructionCommand::Sub));
        assert!(matches!(instructions[4].registers[0], InstructionRegister::A));

        assert!(matches!(instructions[5].variant, InstructionType::SingleReg));
        assert!(matches!(instructions[5].command, InstructionCommand::Inr));
        assert!(matches!(instructions[5].registers[0], InstructionRegister::A));

        assert!(matches!(instructions[6].variant, InstructionType::SingleReg));
        assert!(matches!(instructions[6].command, InstructionCommand::Dcr));
        assert!(matches!(instructions[6].registers[0], InstructionRegister::A));

        assert!(matches!(instructions[7].variant, InstructionType::Intermediate));
        assert!(matches!(instructions[7].command, InstructionCommand::Adi));
        assert_eq!(instructions[7].intermediate, [1,0,0,1,1,0,0,1]);

        assert!(matches!(instructions[8].variant, InstructionType::NoReg));
        assert!(matches!(instructions[8].command, InstructionCommand::Hlt));
    }
}