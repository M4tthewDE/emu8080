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
                && !matches!(InstructionRegister::decode(&raw_instructions[index][2..5]), InstructionRegister::INVALID) &&
                raw_instructions[index][5..] == [1, 1, 0] {

                instruction = Instruction {
                    variant: InstructionType::IntermediateInstruction,
                    command: InstructionCommand::MVI,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][2..5])],
                    intermediate: raw_instructions[index+1].to_vec(),
                };
            // instructions without registers
            // HLT
            } else if raw_instructions[index] == [0, 1, 1, 1, 0, 1, 1, 0] {
                instruction = Instruction {
                    variant: InstructionType::NoRegInstruction,
                    command: InstructionCommand::HLT,
                    registers: Vec::new(),
                    intermediate: Vec::new(),
                };
            // instructions with 1 argument in the end
            // ADD
            } else if raw_instructions[index][0..5] == [1, 0, 0, 0, 0] 
                && !matches!(InstructionRegister::decode(&raw_instructions[index][5..]), InstructionRegister::INVALID) {
                    
                instruction = Instruction {
                    variant: InstructionType::SingleRegInstruction,
                    command: InstructionCommand::ADD,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][5..])],
                    intermediate: Vec::new(),
                }
            // SUB
            } else if raw_instructions[index][0..5] == [1, 0, 0, 1, 0] 
                && !matches!(InstructionRegister::decode(&raw_instructions[index][5..]), InstructionRegister::INVALID) {
                    
                instruction = Instruction {
                    variant: InstructionType::SingleRegInstruction,
                    command: InstructionCommand::SUB,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][5..])],
                    intermediate: Vec::new(),
                }
            // instructions with 1 argument in the middle
            // INR
            } else if raw_instructions[index][0..2] == [0, 0] && raw_instructions[index][5..] == [1, 0, 0] 
                && !matches!(InstructionRegister::decode(&raw_instructions[index][2..5]), InstructionRegister::INVALID) {
                    
                instruction = Instruction {
                    variant: InstructionType::SingleRegInstruction,
                    command: InstructionCommand::INR,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][2..5])],
                    intermediate: Vec::new(),
                }
            // DCR
            } else if raw_instructions[index][0..2] == [0, 0] && raw_instructions[index][5..] == [1, 0, 1] 
                && !matches!(InstructionRegister::decode(&raw_instructions[index][2..5]), InstructionRegister::INVALID) {

                instruction = Instruction {
                    variant: InstructionType::SingleRegInstruction,
                    command: InstructionCommand::DCR,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][2..5])],
                    intermediate: Vec::new(),
                }
            // instructions with 2 registers
            // MOV
            } else if raw_instructions[index][0..2] == [0, 1]
                && !matches!(InstructionRegister::decode(&raw_instructions[index][2..5]), InstructionRegister::INVALID)
                && !matches!(InstructionRegister::decode(&raw_instructions[index][5..]), InstructionRegister::INVALID) {

                let args = vec![
                                InstructionRegister::decode(&raw_instructions[index][2..5]),
                                InstructionRegister::decode(&raw_instructions[index][5..]),
                            ];
                
                instruction = Instruction {
                    variant: InstructionType::DoubleRegInstruction,
                    command: InstructionCommand::MOV,
                    registers: args,
                    intermediate: Vec::new(),
                }
            } else {
                panic!("Invalid instruction!");
            }

            // skip next byte since its the intermediate of the instruction that was just parsed
            if matches!(instruction.variant, InstructionType::IntermediateInstruction) {
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
        assert_eq!(bytes.next().unwrap(), [1,0,0,0,0,1,1,1]);
        assert_eq!(bytes.next().unwrap(), [1,0,0,1,0,1,1,1]);
        assert_eq!(bytes.next().unwrap(), [0,0,1,1,1,1,0,0]);
        assert_eq!(bytes.next().unwrap(), [0,0,1,1,1,1,0,1]);
        assert_eq!(bytes.next().unwrap(), [0,1,1,1,0,1,1,0]);
    }

    #[test]
    fn test_disassemble() {
        let assembler = Assembler::new("test.asm".to_owned(), "output".to_owned());
        assembler.assemble();

        let instructions = assembler.disassemble("output".to_owned());
        assert_eq!(instructions.len(), 7);

        assert!(matches!(instructions[0].variant, InstructionType::IntermediateInstruction));
        assert!(matches!(instructions[0].command, InstructionCommand::MVI));
        assert!(matches!(instructions[0].registers[0], InstructionRegister::A));
        assert_eq!(instructions[0].intermediate, [0,0,0,1,1,1,0,0]);

        assert!(matches!(instructions[1].variant, InstructionType::DoubleRegInstruction));
        assert!(matches!(instructions[1].command, InstructionCommand::MOV));
        assert!(matches!(instructions[1].registers[0], InstructionRegister::A));
        assert!(matches!(instructions[1].registers[1], InstructionRegister::B));

        assert!(matches!(instructions[2].variant, InstructionType::SingleRegInstruction));
        assert!(matches!(instructions[2].command, InstructionCommand::ADD));
        assert!(matches!(instructions[2].registers[0], InstructionRegister::A));

        assert!(matches!(instructions[3].variant, InstructionType::SingleRegInstruction));
        assert!(matches!(instructions[3].command, InstructionCommand::SUB));
        assert!(matches!(instructions[3].registers[0], InstructionRegister::A));

        assert!(matches!(instructions[4].variant, InstructionType::SingleRegInstruction));
        assert!(matches!(instructions[4].command, InstructionCommand::INR));
        assert!(matches!(instructions[4].registers[0], InstructionRegister::A));

        assert!(matches!(instructions[5].variant, InstructionType::SingleRegInstruction));
        assert!(matches!(instructions[5].command, InstructionCommand::DCR));
        assert!(matches!(instructions[5].registers[0], InstructionRegister::A));

        assert!(matches!(instructions[6].variant, InstructionType::NoRegInstruction));
        assert!(matches!(instructions[6].command, InstructionCommand::HLT));
    }
}