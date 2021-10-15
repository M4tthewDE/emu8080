use pest::Parser;
use pest::iterators::Pairs;
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
        // check if its a comment or eoi, since those cant be parsed
        if is_parseable_instruction(raw_instruction.as_rule()) {

            // go one level deeper into pairs, to reach different instruction types
            let raw_instruction = raw_instruction.into_inner().peek().unwrap();
            match raw_instruction.as_rule() {
                Rule::double_reg_instruction => {
                    instructions.push(parse_double_reg_instruction(raw_instruction.into_inner()))
                },
                Rule::single_reg_instruction => {
                    instructions.push(parse_single_reg_instruction(raw_instruction.into_inner()))
                },
                Rule::no_reg_instruction => {
                    instructions.push(parse_no_reg_instruction(raw_instruction.into_inner()))
                },
                Rule::intermediate_reg_instruction => {
                    instructions.push(parse_intermediate_reg_instruction(raw_instruction.into_inner()))
                },
                Rule::intermediate_instruction => {
                    instructions.push(parse_intermediate_instruction(raw_instruction.into_inner()))
                },
                _=> panic!{"invalid rule!"},
            }
        }
    }
    instructions
}

fn is_parseable_instruction(rule: Rule) -> bool {
    match rule {
        Rule::comment => false,
        Rule::EOI => false,
        _ => true
    }
}

fn parse_double_reg_instruction(mut pairs: Pairs<crate::assembler::parser::Rule>) -> Instruction {
    let command = InstructionCommand::from_str(pairs.peek().unwrap().as_str()).unwrap();
    pairs.next();

    let mut args = vec![InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap()];
    pairs.next();
    args.push(InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap());

    Instruction {
        variant: InstructionType::DoubleRegInstruction,
        command,
        registers: args,
        intermediate: Vec::new(),
    }

}

fn parse_single_reg_instruction(mut pairs: Pairs<crate::assembler::parser::Rule>) -> Instruction {
    let command = InstructionCommand::from_str(pairs.peek().unwrap().as_str()).unwrap();
    pairs.next();
    let arg = InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap();

    Instruction {
        variant: InstructionType::SingleRegInstruction,
        command,
        registers: vec![arg],
        intermediate: Vec::new(),
    }
}

fn parse_no_reg_instruction(pairs: Pairs<crate::assembler::parser::Rule>) -> Instruction {
    let command = InstructionCommand::from_str(pairs.peek().unwrap().as_str()).unwrap();

    Instruction {
        variant: InstructionType::NoRegInstruction,
        command,
        registers: Vec::new(),
        intermediate: Vec::new(),
    }
}

fn parse_intermediate_instruction(mut pairs: Pairs<crate::assembler::parser::Rule>) -> Instruction {
    let mut intermediate = Vec::new();

    let command = InstructionCommand::from_str(pairs.peek().unwrap().as_str()).unwrap();
    pairs.next();

    for char in pairs.as_str().chars() {
        if char == '0' {
            intermediate.push(0);
        } else {
            intermediate.push(1);
        }
    }
    Instruction {
        variant: InstructionType::IntermediateInstruction,
        command,
        registers: vec![],
        intermediate,
    }

}

fn parse_intermediate_reg_instruction(mut pairs: Pairs<crate::assembler::parser::Rule>) -> Instruction {
    let mut intermediate = Vec::new();

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
    Instruction {
        variant: InstructionType::IntermediateRegInstruction,
        command,
        registers: vec![arg],
        intermediate,
    }
}    

#[derive(Debug, EnumString)]
pub enum InstructionCommand {
    MVI,
    MOV,
    ADD,
    ADI,
    SUB,
    INR,
    DCR,
    ANA,
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
            _ => panic!("Invalid register"),
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
    IntermediateRegInstruction,
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
            InstructionCommand::ADI => {
                vec![
                    vec![1,1,0,0,0,1,1,0], 
                self.intermediate.clone()]
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
            InstructionCommand::ANA => {
                vec![[
                &[1,0,1,0,0],
                self.registers[0].encode(), 
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

#[cfg(test)]
mod tests {
    use super::InstructionRegister;

    #[test]
    fn test_register_encoding() {
        assert_eq!(InstructionRegister::A.encode(), [1,1,1]);
        assert_eq!(InstructionRegister::B.encode(), [0,0,0]);
        assert_eq!(InstructionRegister::C.encode(), [0,0,1]);
        assert_eq!(InstructionRegister::D.encode(), [0,1,0]);
        assert_eq!(InstructionRegister::E.encode(), [0,1,1]);
        assert_eq!(InstructionRegister::H.encode(), [1,0,0]);
        assert_eq!(InstructionRegister::L.encode(), [1,0,1]);
        assert_eq!(InstructionRegister::M.encode(), [1,1,0]);
    }

    #[test]
    #[should_panic]
    fn test_register_encoding_panic() {
        InstructionRegister::INVALID.encode();
    }

    #[test]
    fn test_register_decoding() {
        assert!(matches!(InstructionRegister::decode(&[1,1,1]), InstructionRegister::A));
        assert!(matches!(InstructionRegister::decode(&[0,0,0]), InstructionRegister::B));
        assert!(matches!(InstructionRegister::decode(&[0,0,1]), InstructionRegister::C));
        assert!(matches!(InstructionRegister::decode(&[0,1,0]), InstructionRegister::D));
        assert!(matches!(InstructionRegister::decode(&[0,1,1]), InstructionRegister::E));
        assert!(matches!(InstructionRegister::decode(&[1,0,0]), InstructionRegister::H));
        assert!(matches!(InstructionRegister::decode(&[1,0,1]), InstructionRegister::L));
        assert!(matches!(InstructionRegister::decode(&[1,1,0]), InstructionRegister::M));
    }

    #[test]
    #[should_panic]
    fn test_register_decoding_panic() {
        InstructionRegister::decode(&[1,1,1,1]);
    }

    #[test]
    fn test_to_index() {
        assert_eq!(InstructionRegister::A.to_index(), 0);
        assert_eq!(InstructionRegister::B.to_index(), 1);
        assert_eq!(InstructionRegister::C.to_index(), 2);
        assert_eq!(InstructionRegister::D.to_index(), 3);
        assert_eq!(InstructionRegister::E.to_index(), 4);
        assert_eq!(InstructionRegister::H.to_index(), 5);
        assert_eq!(InstructionRegister::L.to_index(), 6);
        assert_eq!(InstructionRegister::M.to_index(), 7);
    }

    #[test]
    #[should_panic]
    fn test_register_to_index_panic() {
        InstructionRegister::INVALID.to_index();
    }
}