use std::fs::File;
use std::io::{BufRead, BufReader, Write, Read};
use strum_macros::EnumString;
use std::str::FromStr;

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
        let instructions = self.parse_instructions();

        // write to file
        // TODO actually write hex data instead of binary as ASCII
        let mut file = File::create(&self.output_bin_name).unwrap();
        for instruction in instructions {
            file.write_all(&instruction.encode()).unwrap();
        }
    }

    pub fn disassemble(&self, input_bin: String) {
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

        let instructions = &self.parse_binary_instructions(&raw_instructions);

        println!("{:?}", instructions);
    }

    fn parse_instructions(&self) -> Vec<Instruction> {
        let reader = BufReader::new(&self.input_asm);

        let mut instructions = Vec::new();
        for line in reader.lines() {
            let line = line.unwrap();

            let words: Vec<&str> = line.split(" ").collect();
            let command = InstructionCommand::from_str(words[0]).unwrap();

            if command.has_arguments() {
                instructions.push(Instruction{
                    command: command,
                    arguments: get_instruction_args(words[1]),
                })
            } else {
                instructions.push(Instruction{
                    command: command,
                    arguments: Vec::new(),
                })
            }
        }
        instructions
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
        // instructions without arguments
        // HLT
        if &byte == &[0, 1, 1, 1, 0, 1, 1, 0] {
            Instruction {
                command: InstructionCommand::HLT,
                arguments: Vec::new(),
            }
        // instructions with 1 argument in the end
        // ADD
        } else if &byte[0..5] == &[1, 0, 0, 0, 0] 
            && !matches!(InstructionArgument::decode(&byte[5..]), InstructionArgument::INVALID) {
                
            Instruction {
                command: InstructionCommand::ADD,
                arguments: vec![InstructionArgument::decode(&byte[5..])],
            }
        // SUB
        } else if &byte[0..5] == &[1, 0, 0, 1, 0] 
            && !matches!(InstructionArgument::decode(&byte[5..]), InstructionArgument::INVALID) {
                
            Instruction {
                command: InstructionCommand::SUB,
                arguments: vec![InstructionArgument::decode(&byte[5..])],
            }
        // instructions with 1 argument in the middle
        // INR
        } else if &byte[0..2] == &[0, 0] && &byte[5..] == &[1, 0, 0] 
            && !matches!(InstructionArgument::decode(&byte[2..5]), InstructionArgument::INVALID) {
                
            Instruction {
                command: InstructionCommand::INR,
                arguments: vec![InstructionArgument::decode(&byte[2..5])],
            }
        // DCR
        } else if &byte[0..2] == &[0, 0] && &byte[5..] == &[1, 0, 1] 
            && !matches!(InstructionArgument::decode(&byte[2..5]), InstructionArgument::INVALID) {

            Instruction {
                command: InstructionCommand::DCR,
                arguments: vec![InstructionArgument::decode(&byte[2..5])],
            }
        // instructions with 2 arguments
        // MOV
        } else if &byte[0..2] == &[0, 1]
            && !matches!(InstructionArgument::decode(&byte[2..5]), InstructionArgument::INVALID)
            && !matches!(InstructionArgument::decode(&byte[5..]), InstructionArgument::INVALID) {

            let mut args = Vec::new();
            args.push(InstructionArgument::decode(&byte[2..5]));
            args.push(InstructionArgument::decode(&byte[5..]));
            
            Instruction {
                command: InstructionCommand::MOV,
                arguments: args,
            }
        } else {
            panic!("Invalid instruction!");
        }
    }
}

fn get_instruction_args(word: &str) -> Vec<InstructionArgument> {
    let raw_args: Vec<&str> = word.split(",").collect();

    let mut args = Vec::new();
    for raw_arg in raw_args {
        let arg = InstructionArgument::from_str(raw_arg).unwrap();
        args.push(arg);
    }

    args
}

#[derive(Debug)]
struct Instruction {
    command: InstructionCommand,
    arguments: Vec<InstructionArgument>,
}

impl Instruction {
    pub fn encode(&self) -> Vec<u8> {
        match self.command {
            InstructionCommand::MOV => {
                [
                &[0,1], 
                self.arguments[0].encode(), 
                self.arguments[1].encode(),
                ].concat()
            },
            InstructionCommand::ADD => {
                [
                &[1,0,0,0,0],
                self.arguments[0].encode(), 
                ].concat()
            },
            InstructionCommand::SUB => {
                [
                &[1,0,0,1,0],
                self.arguments[0].encode(), 
                ].concat()
            },
            InstructionCommand::INR => {
                [
                &[0,0],
                self.arguments[0].encode(), 
                &[1,0,0],
                ].concat()
            },
            InstructionCommand::DCR => {
                [
                &[0,0],
                self.arguments[0].encode(), 
                &[1,0,1],
                ].concat()
            },
            InstructionCommand::HLT => {
                vec!(0,1,1,1,0,1,1,0)
            },
        }
    }
}

#[derive(Debug, EnumString)]
enum InstructionCommand {
    MOV,
    ADD,
    SUB,
    INR,
    DCR,
    HLT,
}

impl InstructionCommand {
    pub fn has_arguments(&self) -> bool {
        match self {
            InstructionCommand::HLT => false,
            _ => true,
        }
    }
}

#[derive(Debug, EnumString)]
enum InstructionArgument {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    M,
    INVALID,
}

impl InstructionArgument {
    pub fn encode(&self) -> &[u8]{
        match self {
            InstructionArgument::A => &[1,1,1],
            InstructionArgument::B => &[0,0,0],
            InstructionArgument::C => &[0,0,1],
            InstructionArgument::D => &[0,1,0],
            InstructionArgument::E => &[0,1,1],
            InstructionArgument::H => &[1,0,0],
            InstructionArgument::L => &[1,0,1],
            InstructionArgument::M => &[1,1,0],
            _ => panic!("Invalid argument provided")
        }
    }

    pub fn decode(raw_bytes: &[u8]) -> InstructionArgument {
        match raw_bytes {
            &[1,1,1] => InstructionArgument::A,
            &[0,0,0] => InstructionArgument::B,
            &[0,0,1] => InstructionArgument::C,
            &[0,1,0] => InstructionArgument::D,
            &[0,1,1] => InstructionArgument::E,
            &[1,0,0] => InstructionArgument::H,
            &[1,0,1] => InstructionArgument::L,
            &[1,1,0] => InstructionArgument::M,
            _ => InstructionArgument::INVALID,
        }
    }
}