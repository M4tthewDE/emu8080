use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use strum_macros::EnumString;
use std::str::FromStr;

#[derive(Debug)]
pub struct Assembler{
    input_asm: File,
    output_bin_name: String,
}

impl Assembler {
    pub fn new(input_asm_name: String, output_bin_name: String) -> Assembler {
        let input = File::open(input_asm_name);
        let input = match input {
            Ok(input) => input,
            Err(error) => panic!("Problem opening file: {:?}", error),
        };

        Assembler {
            input_asm: input,
            output_bin_name: output_bin_name,
        }
    }

    pub fn assemble(&self) {
        let instructions = self.parse_instructions();

        // write in hex to file
        let mut file = File::create(&self.output_bin_name).unwrap();
        for instruction in instructions {
            file.write_all(&instruction.get_encoding()).unwrap();
        }
    }

    fn parse_instructions(&self) -> Vec<Instruction> {
        let reader = BufReader::new(&self.input_asm);

        let mut instructions = Vec::new();
        for (_, line) in reader.lines().enumerate() {
            let line = line.unwrap();

            let words: Vec<&str> = line.split(" ").collect();
            let command = InstructionCommand::from_str(words[0]).unwrap();

            if command.has_arguments() {
                let args = get_instruction_args(words[1]);
                instructions.push(Instruction{
                    command: command,
                    arguments: args,
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
    pub fn get_encoding(&self) -> Vec<u8> {
        match self.command {
            InstructionCommand::MOV => {
                [
                b"01", 
                self.arguments[0].get_encoding(), 
                self.arguments[1].get_encoding(),
                ].concat()
            },
            InstructionCommand::ADD => {
                [
                b"10000", 
                self.arguments[0].get_encoding(), 
                ].concat()
            },
            InstructionCommand::SUB => {
                [
                b"10010", 
                self.arguments[0].get_encoding(), 
                ].concat()
            },
            InstructionCommand::INR => {
                [
                b"00", 
                self.arguments[0].get_encoding(), 
                b"100",
                ].concat()
            },
            InstructionCommand::DCR => {
                [
                b"00", 
                self.arguments[0].get_encoding(), 
                b"101",
                ].concat()
            },
            InstructionCommand::HLT => {
                b"01110110".to_vec()
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
}

impl InstructionArgument {
    pub fn get_encoding(&self) -> &[u8]{
        match self {
            InstructionArgument::A => b"111",
            InstructionArgument::B => b"000",
            InstructionArgument::C => b"001",
            InstructionArgument::D => b"010",
            InstructionArgument::E => b"011",
            InstructionArgument::H => b"100",
            InstructionArgument::L => b"101",
            InstructionArgument::M => b"110",
        }
    }
}