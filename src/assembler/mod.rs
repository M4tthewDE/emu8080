use std::fs::File;
use std::io::{BufRead, BufReader};
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
        println!("{:?}", instructions);
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

#[derive(Debug, EnumString)]
enum InstructionCommand {
    MOV,
    ADD,
    SUB,
    INCR,
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
    D,
    S,
}