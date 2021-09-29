use std::fs::File;
use std::io::{Write, Read};
use crate::assembler::parser::Encoding;
pub use crate::assembler::parser::{InstructionRegister, Instruction, InstructionCommand, InstructionType};

mod parser;

#[derive(Debug)]
pub struct Assembler{
    input_asm: File,
    output_bin_name: String,
}

impl Assembler {
    pub fn new(input_asm_name: String, output_bin_name: String) -> Assembler {
        let input = File::open(input_asm_name).unwrap();

        Assembler {
            input_asm: input,
            output_bin_name: output_bin_name,
        }
    }

    pub fn assemble(&self) {
        let instructions = parser::parse();

        // write to file
        // TODO actually write hex data instead of binary as ASCII
        let mut file = File::create(&self.output_bin_name).unwrap();
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

    fn parse_binary_instructions(&self, raw_instructions: &Vec<&[u8]>) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        for raw_instruction in raw_instructions {
            let instruction = self.decode_raw_intructions(raw_instruction);
            instructions.push(instruction);
        }

        instructions
    }

    fn decode_raw_intructions(&self, byte: &[u8]) -> Instruction {
        // pretty ugly, maybe there is a better solution with match or something
        // instructions without registers
        // HLT
        if &byte == &[0, 1, 1, 1, 0, 1, 1, 0] {
            Instruction {
                variant: InstructionType::NoRegInstruction,
                command: InstructionCommand::HLT,
                registers: Vec::new(),
                intermediate: 0,
            }
        // instructions with 1 argument in the end
        // ADD
        } else if &byte[0..5] == &[1, 0, 0, 0, 0] 
            && !matches!(InstructionRegister::decode(&byte[5..]), InstructionRegister::INVALID) {
                
            Instruction {
                variant: InstructionType::SingleRegInstruction,
                command: InstructionCommand::ADD,
                registers: vec![InstructionRegister::decode(&byte[5..])],
                intermediate: 0,
            }
        // SUB
        } else if &byte[0..5] == &[1, 0, 0, 1, 0] 
            && !matches!(InstructionRegister::decode(&byte[5..]), InstructionRegister::INVALID) {
                
            Instruction {
                variant: InstructionType::SingleRegInstruction,
                command: InstructionCommand::SUB,
                registers: vec![InstructionRegister::decode(&byte[5..])],
                intermediate: 0,
            }
        // instructions with 1 argument in the middle
        // INR
        } else if &byte[0..2] == &[0, 0] && &byte[5..] == &[1, 0, 0] 
            && !matches!(InstructionRegister::decode(&byte[2..5]), InstructionRegister::INVALID) {
                
            Instruction {
                variant: InstructionType::SingleRegInstruction,
                command: InstructionCommand::INR,
                registers: vec![InstructionRegister::decode(&byte[2..5])],
                intermediate: 0,
            }
        // DCR
        } else if &byte[0..2] == &[0, 0] && &byte[5..] == &[1, 0, 1] 
            && !matches!(InstructionRegister::decode(&byte[2..5]), InstructionRegister::INVALID) {

            Instruction {
                variant: InstructionType::SingleRegInstruction,
                command: InstructionCommand::DCR,
                registers: vec![InstructionRegister::decode(&byte[2..5])],
                intermediate: 0,
            }
        // instructions with 2 registers
        // MOV
        } else if &byte[0..2] == &[0, 1]
            && !matches!(InstructionRegister::decode(&byte[2..5]), InstructionRegister::INVALID)
            && !matches!(InstructionRegister::decode(&byte[5..]), InstructionRegister::INVALID) {

            let mut args = Vec::new();
            args.push(InstructionRegister::decode(&byte[2..5]));
            args.push(InstructionRegister::decode(&byte[5..]));
            
            Instruction {
                variant: InstructionType::DoubleRegInstruction,
                command: InstructionCommand::MOV,
                registers: args,
                intermediate: 0,
            }
        } else {
            panic!("Invalid instruction!");
        }
    }
}
