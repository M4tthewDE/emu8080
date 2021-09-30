use pest::Parser;
use std::fs;
use strum_macros::EnumString;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "asm.pest"]
pub struct AssemblyParser;

pub fn parse(file_name: String) -> Vec<Instruction> {
    let unparsed_file = fs::read_to_string(file_name).unwrap();

    let file = AssemblyParser::parse(Rule::assembly, &unparsed_file).expect("unsuccessful parse").next().unwrap();

    let mut instructions = Vec::new();
    for raw_instruction in file.into_inner() {
        match raw_instruction.as_rule() {
            Rule::double_reg_instruction => {
                let mut pairs = raw_instruction.into_inner();

                let command = InstructionCommand::from_str(pairs.peek().unwrap().as_str()).unwrap();
                pairs.next();

                let mut args = vec![InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap()];
                pairs.next();
                args.push(InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap());

                instructions.push(
                    Instruction {
                        variant: InstructionType::DoubleRegInstruction,
                        command,
                        registers: args,
                        intermediate: Vec::new(),
                    }
                )
            },
            Rule::single_reg_instruction => {
                let mut pairs = raw_instruction.into_inner();

                let command = InstructionCommand::from_str(pairs.peek().unwrap().as_str()).unwrap();
                pairs.next();
                let arg = InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap();

                instructions.push(
                    Instruction {
                        variant: InstructionType::SingleRegInstruction,
                        command,
                        registers: vec![arg],
                        intermediate: Vec::new(),
                    }
                )
            },
            Rule::no_reg_instruction => {
                let pairs = raw_instruction.into_inner();

                let command = InstructionCommand::from_str(pairs.peek().unwrap().as_str()).unwrap();
                instructions.push(
                    Instruction {
                        variant: InstructionType::NoRegInstruction,
                        command,
                        registers: Vec::new(),
                        intermediate: Vec::new(),
                    }
                )
            },
            Rule::intermediate_instruction => {
                let mut intermediate = Vec::new();
                let mut pairs = raw_instruction.into_inner();

                let command = InstructionCommand::from_str(pairs.peek().unwrap().as_str()).unwrap();
                pairs.next();

                let arg = InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap();
                pairs.next();

                for char in pairs.as_str().chars() {
                    if char == '0' {
                        intermediate.push(0);
                    } else {
                        intermediate.push(1);
                    }
                }
                instructions.push(
                    Instruction {
                        variant: InstructionType::IntermediateInstruction,
                        command,
                        registers: vec![arg],
                        intermediate,
                    }
                )
            },
            _=> panic!{"invalid rule!"},
        }
    }
    instructions
}

#[derive(Debug, EnumString)]
pub enum InstructionCommand {
    MVI,
    MOV,
    ADD,
    SUB,
    INR,
    DCR,
    HLT,
}

#[derive(Debug, EnumString)]
pub enum InstructionRegister {
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

impl InstructionRegister {
    pub fn encode(&self) -> &[u8]{
        match self {
            InstructionRegister::A => &[1,1,1],
            InstructionRegister::B => &[0,0,0],
            InstructionRegister::C => &[0,0,1],
            InstructionRegister::D => &[0,1,0],
            InstructionRegister::E => &[0,1,1],
            InstructionRegister::H => &[1,0,0],
            InstructionRegister::L => &[1,0,1],
            InstructionRegister::M => &[1,1,0],
            InstructionRegister::INVALID => {panic!("invalid register")},
        }
    }

    pub fn decode(raw_bytes: &[u8]) -> InstructionRegister {
        match *raw_bytes {
            [1,1,1] => InstructionRegister::A,
            [0,0,0] => InstructionRegister::B,
            [0,0,1] => InstructionRegister::C,
            [0,1,0] => InstructionRegister::D,
            [0,1,1] => InstructionRegister::E,
            [1,0,0] => InstructionRegister::H,
            [1,0,1] => InstructionRegister::L,
            [1,1,0] => InstructionRegister::M,
            _ => panic!("Invalid invalid register"),
        }
    }
    
    pub fn to_index(&self) -> u8 {
        match self {
            InstructionRegister::A => 0,
            InstructionRegister::B => 1,
            InstructionRegister::C => 2,
            InstructionRegister::D => 3,
            InstructionRegister::E => 4,
            InstructionRegister::H => 5,
            InstructionRegister::L => 6,
            InstructionRegister::M => 7,
            _ => panic!("Invalid argument provided!")
        }
    }
}

pub trait Encoding {
    fn encode(&self) -> Vec<Vec<u8>>;
}

#[derive(Debug)]
pub enum InstructionType {
    NoRegInstruction,
    SingleRegInstruction,
    DoubleRegInstruction,
    IntermediateInstruction,
}

#[derive(Debug)]
pub struct Instruction {
    pub variant: InstructionType,
    pub command: InstructionCommand,
    pub registers: Vec<InstructionRegister>,
    pub intermediate: Vec<u8>,
}

impl Encoding for Instruction {
    fn encode(&self) -> Vec<Vec<u8>> {
        match self.command {
            InstructionCommand::MVI => {
                vec![[
                &[0,0],
                self.registers[0].encode(),
                &[1,1,0]
                ].concat(), self.intermediate.clone()]
            }
            InstructionCommand::ADD => {
                vec![[
                &[1,0,0,0,0],
                self.registers[0].encode(), 
                ].concat()]
            },
            InstructionCommand::SUB => {
                vec![[
                &[1,0,0,1,0],
                self.registers[0].encode(), 
                ].concat()]
            },
            InstructionCommand::INR => {
                vec![[
                &[0,0],
                self.registers[0].encode(), 
                &[1,0,0],
                ].concat()]
            },
            InstructionCommand::DCR => {
                vec![[
                &[0,0],
                self.registers[0].encode(), 
                &[1,0,1],
                ].concat()]
            },
            InstructionCommand::MOV => {
                vec![[
                &[0,1], 
                self.registers[0].encode(), 
                self.registers[1].encode(),
                ].concat()]
            },
            InstructionCommand::HLT => {vec![vec![0,1,1,1,0,1,1,0]]},
        }
    }
}
