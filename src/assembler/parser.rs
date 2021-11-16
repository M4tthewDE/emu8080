use pest::Parser;
use std::fs;
use std::str::FromStr;
use strum_macros::EnumString;

#[derive(Parser)]
#[grammar = "asm.pest"]
pub struct AssemblyParser;

pub fn parse(file_name: String) -> (Vec<Instruction>, Vec<Label>) {
    let unparsed_file = fs::read_to_string(file_name).unwrap();

    let assembly = AssemblyParser::parse(Rule::assembly, &unparsed_file)
        .expect("unsuccessful parse")
        .next()
        .unwrap();

    let raw_instructions = assembly.into_inner();
    println!("{:?}", raw_instructions);

    let mut instructions = Vec::new();
    let mut labels = Vec::new();
    let mut label_position = 0;

    for instruction in raw_instructions {
        let rule = instruction.as_rule();

        // ignore comments and end of input
        if !matches!(rule, Rule::comment | Rule::EOI) {
            let mut inner_instruction_pairs = instruction.into_inner();
            let inner_instruction = inner_instruction_pairs.peek().unwrap();

            let mut rule = inner_instruction.as_rule();

            if matches!(rule, Rule::label) {
                let name = inner_instruction.as_str().to_string();

                let label = Label {
                    name: name[0..name.len() - 1].to_string(),
                    position: label_position,
                };

                if labels.contains(&label) {
                    panic!("can't have duplicate labels: {:?}", label);
                } else if InstructionCommand::from_str(&label.name).is_ok()
                    || InstructionRegister::from_str(&label.name).is_ok()
                {
                    panic!("label can't occupy reserved names: {:?}", label);
                }

                labels.push(label);

                inner_instruction_pairs.next();
            }

            let mut pairs = inner_instruction_pairs.peek().unwrap().into_inner();
            let inner_instruction = pairs.peek().unwrap();
            rule = inner_instruction.as_rule();

            let command = InstructionCommand::from_str(inner_instruction.as_str()).unwrap();
            pairs.next();

            match rule {
                Rule::intermediate_reg_command => {
                    let registers =
                        vec![
                            InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap(),
                        ];
                    pairs.next();

                    let mut intermediate = Vec::new();
                    for char in pairs.as_str().chars() {
                        if char == '0' {
                            intermediate.push(0);
                        } else {
                            intermediate.push(1);
                        }
                    }

                    let instruction = Instruction {
                        variant: InstructionType::IntermediateReg,
                        command,
                        registers,
                        intermediate,
                    };

                    instructions.push(instruction);
                }
                Rule::double_reg_command => {
                    let mut registers =
                        vec![
                            InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap(),
                        ];
                    pairs.next();
                    registers.push(
                        InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap(),
                    );
                    pairs.next();

                    let instruction = Instruction {
                        variant: InstructionType::DoubleReg,
                        command,
                        registers,
                        intermediate: Vec::new(),
                    };
                    instructions.push(instruction);
                }
                Rule::single_reg_command => {
                    let registers =
                        vec![
                            InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap(),
                        ];
                    pairs.next();

                    let instruction = Instruction {
                        variant: InstructionType::SingleReg,
                        command,
                        registers,
                        intermediate: Vec::new(),
                    };
                    instructions.push(instruction);
                }
                Rule::intermediate_command => {
                    let mut intermediate = Vec::new();
                    for char in pairs.as_str().chars() {
                        if char == '0' {
                            intermediate.push(0);
                        } else {
                            intermediate.push(1);
                        }
                    }

                    let instruction = Instruction {
                        variant: InstructionType::IntermediateReg,
                        command,
                        registers: Vec::new(),
                        intermediate,
                    };
                    instructions.push(instruction);
                }
                Rule::no_reg_command => {
                    let instruction = Instruction {
                        variant: InstructionType::NoReg,
                        command,
                        registers: Vec::new(),
                        intermediate: Vec::new(),
                    };
                    instructions.push(instruction);
                }
                _ => panic!("invalid rule: {:?}", rule),
            }
            label_position += 1;
        }
    }
    (instructions, labels)
}

#[derive(Debug)]
pub struct Label {
    pub name: String,
    pub position: usize,
}

impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Debug, EnumString)]
pub enum InstructionCommand {
    #[strum(serialize = "MVI")]
    Mvi,
    #[strum(serialize = "MOV")]
    Mov,
    #[strum(serialize = "ADD")]
    Add,
    #[strum(serialize = "ADC")]
    Adc,
    #[strum(serialize = "ADI")]
    Adi,
    #[strum(serialize = "ACI")]
    Aci,
    #[strum(serialize = "SUI")]
    Sui,
    #[strum(serialize = "SUB")]
    Sub,
    #[strum(serialize = "INR")]
    Inr,
    #[strum(serialize = "DCR")]
    Dcr,
    #[strum(serialize = "ANA")]
    Ana,
    #[strum(serialize = "STC")]
    Stc,
    #[strum(serialize = "CMC")]
    Cmc,
    #[strum(serialize = "CMA")]
    Cma,
    #[strum(serialize = "RLC")]
    Rlc,
    #[strum(serialize = "RRC")]
    Rrc,
    #[strum(serialize = "RAL")]
    Ral,
    #[strum(serialize = "RAR")]
    Rar,
    #[strum(serialize = "ORA")]
    Ora,
    #[strum(serialize = "DAA")]
    Daa,
    #[strum(serialize = "HLT")]
    Hlt,
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
    Invalid,
}

impl InstructionRegister {
    pub fn encode(&self) -> &[u8] {
        match self {
            InstructionRegister::A => &[1, 1, 1],
            InstructionRegister::B => &[0, 0, 0],
            InstructionRegister::C => &[0, 0, 1],
            InstructionRegister::D => &[0, 1, 0],
            InstructionRegister::E => &[0, 1, 1],
            InstructionRegister::H => &[1, 0, 0],
            InstructionRegister::L => &[1, 0, 1],
            InstructionRegister::M => &[1, 1, 0],
            InstructionRegister::Invalid => {
                panic!("invalid register")
            }
        }
    }

    pub fn decode(raw_bytes: &[u8]) -> InstructionRegister {
        match *raw_bytes {
            [1, 1, 1] => InstructionRegister::A,
            [0, 0, 0] => InstructionRegister::B,
            [0, 0, 1] => InstructionRegister::C,
            [0, 1, 0] => InstructionRegister::D,
            [0, 1, 1] => InstructionRegister::E,
            [1, 0, 0] => InstructionRegister::H,
            [1, 0, 1] => InstructionRegister::L,
            [1, 1, 0] => InstructionRegister::M,
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
            _ => panic!("Invalid argument provided!"),
        }
    }
}

pub trait Encoding {
    fn encode(&self) -> Vec<Vec<u8>>;
}

#[derive(Debug)]
pub enum InstructionType {
    NoReg,
    SingleReg,
    DoubleReg,
    IntermediateReg,
    Intermediate,
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
            InstructionCommand::Mvi => {
                vec![
                    [&[0, 0], self.registers[0].encode(), &[1, 1, 0]].concat(),
                    self.intermediate.clone(),
                ]
            }
            InstructionCommand::Adi => {
                vec![vec![1, 1, 0, 0, 0, 1, 1, 0], self.intermediate.clone()]
            }
            InstructionCommand::Aci => {
                vec![vec![1, 1, 0, 0, 1, 1, 1, 0], self.intermediate.clone()]
            }
            InstructionCommand::Sui => {
                vec![vec![1, 1, 0, 1, 0, 1, 1, 0], self.intermediate.clone()]
            }
            InstructionCommand::Add => {
                vec![[&[1, 0, 0, 0, 0], self.registers[0].encode()].concat()]
            }
            InstructionCommand::Adc => {
                vec![[&[1, 0, 0, 0, 1], self.registers[0].encode()].concat()]
            }
            InstructionCommand::Sub => {
                vec![[&[1, 0, 0, 1, 0], self.registers[0].encode()].concat()]
            }
            InstructionCommand::Inr => {
                vec![[&[0, 0], self.registers[0].encode(), &[1, 0, 0]].concat()]
            }
            InstructionCommand::Dcr => {
                vec![[&[0, 0], self.registers[0].encode(), &[1, 0, 1]].concat()]
            }
            InstructionCommand::Ana => {
                vec![[&[1, 0, 1, 0, 0], self.registers[0].encode()].concat()]
            }
            InstructionCommand::Mov => {
                vec![[
                    &[0, 1],
                    self.registers[0].encode(),
                    self.registers[1].encode(),
                ]
                .concat()]
            }
            InstructionCommand::Stc => {
                vec![vec![0, 0, 1, 1, 0, 1, 1, 1]]
            }
            InstructionCommand::Cmc => {
                vec![vec![0, 0, 1, 1, 1, 1, 1, 1]]
            }
            InstructionCommand::Cma => {
                vec![vec![0, 0, 1, 0, 1, 1, 1, 1]]
            }
            InstructionCommand::Rlc => {
                vec![vec![0, 0, 0, 0, 0, 1, 1, 1]]
            }
            InstructionCommand::Rrc => {
                vec![vec![0, 0, 0, 0, 1, 1, 1, 1]]
            }
            InstructionCommand::Ral => {
                vec![vec![0, 0, 0, 1, 0, 1, 1, 1]]
            }
            InstructionCommand::Rar => {
                vec![vec![0, 0, 0, 1, 1, 1, 1, 1]]
            }
            InstructionCommand::Ora => {
                vec![[&[1, 0, 1, 1, 0], self.registers[0].encode()].concat()]
            }
            InstructionCommand::Daa => {
                vec![vec![0, 0, 1, 0, 0, 1, 1, 1]]
            }
            InstructionCommand::Hlt => {
                vec![vec![0, 1, 1, 1, 0, 1, 1, 0]]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InstructionRegister;

    #[test]
    fn test_register_encoding() {
        assert_eq!(InstructionRegister::A.encode(), [1, 1, 1]);
        assert_eq!(InstructionRegister::B.encode(), [0, 0, 0]);
        assert_eq!(InstructionRegister::C.encode(), [0, 0, 1]);
        assert_eq!(InstructionRegister::D.encode(), [0, 1, 0]);
        assert_eq!(InstructionRegister::E.encode(), [0, 1, 1]);
        assert_eq!(InstructionRegister::H.encode(), [1, 0, 0]);
        assert_eq!(InstructionRegister::L.encode(), [1, 0, 1]);
        assert_eq!(InstructionRegister::M.encode(), [1, 1, 0]);
    }

    #[test]
    #[should_panic]
    fn test_register_encoding_panic() {
        InstructionRegister::Invalid.encode();
    }

    #[test]
    fn test_register_decoding() {
        assert!(matches!(
            InstructionRegister::decode(&[1, 1, 1]),
            InstructionRegister::A
        ));
        assert!(matches!(
            InstructionRegister::decode(&[0, 0, 0]),
            InstructionRegister::B
        ));
        assert!(matches!(
            InstructionRegister::decode(&[0, 0, 1]),
            InstructionRegister::C
        ));
        assert!(matches!(
            InstructionRegister::decode(&[0, 1, 0]),
            InstructionRegister::D
        ));
        assert!(matches!(
            InstructionRegister::decode(&[0, 1, 1]),
            InstructionRegister::E
        ));
        assert!(matches!(
            InstructionRegister::decode(&[1, 0, 0]),
            InstructionRegister::H
        ));
        assert!(matches!(
            InstructionRegister::decode(&[1, 0, 1]),
            InstructionRegister::L
        ));
        assert!(matches!(
            InstructionRegister::decode(&[1, 1, 0]),
            InstructionRegister::M
        ));
    }

    #[test]
    #[should_panic]
    fn test_register_decoding_panic() {
        InstructionRegister::decode(&[1, 1, 1, 1]);
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
        InstructionRegister::Invalid.to_index();
    }
}
