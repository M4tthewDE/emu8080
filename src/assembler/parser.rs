use pest::Parser;
use std::convert::TryFrom;
use std::fs;
use std::str::FromStr;
use strum_macros::EnumString;
use std::any::Any;

#[derive(Parser)]
#[grammar = "asm.pest"]
pub struct AssemblyParser;

pub fn parse(file_name: String) -> (Vec<Box<Instruction>>, Vec<Label>) {
    let unparsed_file = fs::read_to_string(file_name).unwrap();

    let assembly = AssemblyParser::parse(Rule::assembly, &unparsed_file)
        .expect("unsuccessful parse")
        .next()
        .unwrap();

    let raw_instructions = assembly.into_inner();

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
                        intermediate: binary_to_int(&mut intermediate),
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
                        intermediate: 0,
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
                        intermediate: 0,
                    };
                    instructions.push(instruction);
                }
                Rule::pair_reg_command => {
                    let registers =
                        vec![
                            InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap(),
                        ];
                    pairs.next();

                    let instruction = Instruction {
                        variant: InstructionType::PairReg,
                        command,
                        registers,
                        intermediate: 0,
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
                        intermediate: binary_to_int(&mut intermediate),
                    };
                    instructions.push(instruction);
                }
                Rule::no_reg_command => {
                    let instruction = Instruction {
                        variant: InstructionType::NoReg,
                        command,
                        registers: Vec::new(),
                        intermediate: 0,
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
    #[strum(serialize = "STAX")]
    Stax,
    #[strum(serialize = "LDAX")]
    Ldax,
    #[strum(serialize = "CMP")]
    Cmp,
    #[strum(serialize = "XRA")]
    Xra,
    #[strum(serialize = "SBB")]
    Sbb,
    #[strum(serialize = "XCHG")]
    Xchg,
    #[strum(serialize = "SPHL")]
    Sphl,
    #[strum(serialize = "XTHL")]
    Xthl,
    #[strum(serialize = "HLT")]
    Hlt,
}

pub trait InstructionArgument {
    fn encode(&self) -> &[u8];
    fn decode(raw_bits: &[u8]) -> Self;
}

#[derive(Debug, Copy, Clone, EnumString)]
pub enum InstructionRegister {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    M,
}

impl InstructionArgument for InstructionRegister {
    fn encode(&self) -> &[u8] {
        match self {
            InstructionRegister::A => &[1, 1, 1],
            InstructionRegister::B => &[0, 0, 0],
            InstructionRegister::C => &[0, 0, 1],
            InstructionRegister::D => &[0, 1, 0],
            InstructionRegister::E => &[0, 1, 1],
            InstructionRegister::H => &[1, 0, 0],
            InstructionRegister::L => &[1, 0, 1],
            InstructionRegister::M => &[1, 1, 0],
        }
    }

    fn decode(raw_bits: &[u8]) -> InstructionRegister {
        match *raw_bits {
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
}

impl InstructionRegister {
    pub fn to_index(self) -> u8 {
        match self {
            InstructionRegister::A => 0,
            InstructionRegister::B => 1,
            InstructionRegister::C => 2,
            InstructionRegister::D => 3,
            InstructionRegister::E => 4,
            InstructionRegister::H => 5,
            InstructionRegister::L => 6,
            InstructionRegister::M => 7,
        }
    }

    pub fn from_index(index: i32) -> InstructionRegister {
        match index {
            0 => InstructionRegister::A,
            1 => InstructionRegister::B,
            2 => InstructionRegister::C,
            3 => InstructionRegister::D,
            4 => InstructionRegister::E,
            5 => InstructionRegister::H,
            6 => InstructionRegister::L,
            7 => InstructionRegister::M,
            _ => panic!("Invalid argument provided!"),
        }
    }
}

pub enum InstructionRegisterPair {
    BC,
    DE,
    HL,
    SP,
}

impl InstructionArgument for InstructionRegisterPair {
    fn encode(&self) -> &[u8] {
        match self {
            InstructionRegisterPair::BC => &[0, 0],
            InstructionRegisterPair::DE => &[0, 1],
            InstructionRegisterPair::HL => &[1, 0],
            InstructionRegisterPair::SP => &[1, 1],
        }
    }

    fn decode(raw_bits: &[u8]) -> InstructionRegisterPair {
        match *raw_bits {
            [0,0] => InstructionRegisterPair::BC,
            [0,1] => InstructionRegisterPair::DE,
            [1,0] => InstructionRegisterPair::HL,
            [1,1] => InstructionRegisterPair::SP,
            _ => panic!("Invalid registerpair"),
        }
    }
}

pub enum InstructionType {
    NoReg,
    SingleReg,
    DoubleReg,
    Intermediate,
    IntermediateReg,
    PairReg,
}

#[derive(Sized)]
pub trait Instruction {
    fn encode(&self) -> Vec<u8>;
    fn get_type(&self) -> InstructionType
    fn as_any(&self) -> &dyn Any;
}

pub struct NoRegInstruction {
    pub command: InstructionCommand
}

impl Instruction for NoRegInstruction {
    fn encode(&self) -> Vec<u8> {
        match self.command {
            InstructionCommand::Stc => {
                vec![0, 0, 1, 1, 0, 1, 1, 1]
            }
            InstructionCommand::Cmc => {
                vec![0, 0, 1, 1, 1, 1, 1, 1]
            }
            InstructionCommand::Cma => {
                vec![0, 0, 1, 0, 1, 1, 1, 1]
            }
            InstructionCommand::Rlc => {
                vec![0, 0, 0, 0, 0, 1, 1, 1]
            }
            InstructionCommand::Rrc => {
                vec![0, 0, 0, 0, 1, 1, 1, 1]
            }
            InstructionCommand::Ral => {
                vec![0, 0, 0, 1, 0, 1, 1, 1]
            }
            InstructionCommand::Rar => {
                vec![0, 0, 0, 1, 1, 1, 1, 1]
            }
        }    
    }

    fn get_type(&self) -> InstructionType {
        InstructionType::NoReg
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
} 

pub struct SingleRegInstruction {
    pub command: InstructionCommand,
    pub register: InstructionRegister,
}

impl Instruction for SingleRegInstruction {
    fn encode(&self) -> Vec<u8> {
        match self.command {
            InstructionCommand::Add => {
                [1, 0, 0, 0, 0].append(self.register.encode())
            }
            InstructionCommand::Adc => {
                [1, 0, 0, 0, 1].append(self.register.encode())
            }
            InstructionCommand::Sub => {
                [1, 0, 0, 1, 0].append(self.register.encode())
            }
            InstructionCommand::Inr => {
                [0, 0].append(self.register.encode()).append([1,0,0])
            }
            InstructionCommand::Dcr => {
                [0, 0].append(self.register.encode()).append([1,0,1])
            }
            InstructionCommand::Ana => {
                [1, 0, 1, 0, 0].append(self.register.encode())
            }
        }    
    }

    fn get_type(&self) -> InstructionType {
        InstructionType::SingleReg
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
} 

pub struct DoubleRegInstruction {
    pub command: InstructionCommand,
    pub registers: (InstructionRegister, InstructionRegister),
}

impl Instruction for DoubleRegInstruction {
    fn encode(&self) -> Vec<u8> {
        match self.command {
            InstructionCommand::Mov => {
                    &[0, 1]
                        .append(self.registers[0].encode())
                        .append(self.registers[1].encode())
            }
        }    
    }

    fn get_type(&self) -> InstructionType {
        InstructionType::DoubleReg
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
} 

pub struct IntermediateInstruction {
    pub command: InstructionCommand,
    pub intermediate: i8,
}

impl Instruction for IntermediateInstruction {
    fn encode(&self) -> Vec<u8> {
        match self.command {
            InstructionCommand::Adi => {
                [1, 1, 0, 0, 0, 1, 1, 0]
                .append(int_to_binary(self.intermediate))
            }
            InstructionCommand::Aci => {
                [1, 1, 0, 0, 1, 1, 1, 0],
                .append(int_to_binary(self.intermediate))
            }
            InstructionCommand::Sui => {
                [1, 1, 0, 1, 0, 1, 1, 0],
                .append(int_to_binary(self.intermediate))
            }
        }    
    }

    fn get_type(&self) -> InstructionType {
        InstructionType::Intermediate
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
} 

pub struct IntermediateRegInstruction {
    pub command: InstructionCommand,
    pub register: InstructionRegister,
    pub intermediate: i8,
}

impl Instruction for IntermediateRegInstruction {
    fn encode(&self) -> Vec<u8> {
        match self.command {
            InstructionCommand::Mvi => {
                [0, 0]
                .append(self.registers[0].encode())
                .append([1, 1, 0])
                .append(int_to_binary(self.intermediate))
            }
        }    
    }

    fn get_type(&self) -> InstructionType {
        InstructionType::IntermediateReg
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
} 

pub struct PairRegInstruction {
    pub command: InstructionCommand,
    pub register_pair: InstructionRegisterPair
}

impl Instruction for PairRegInstruction {
    fn encode(&self) -> Vec<u8> {
        match self.command {
            InstructionCommand::Stax => match self.registers[0] {
                [0,0]
                .append(self.register_pair.encode())
                .append([0,0,1,0])
            },
            InstructionCommand::Ldax => match self.registers[0] {
                [0,0]
                .append(self.register_pair.encode())
                .append([1,0,1,0])
            },
        }    
    }

    fn get_type(&self) -> InstructionType {
        InstructionType::PairReg
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
} 

fn int_to_binary(value: i8) -> Vec<u8> {
    let binary_string = format!("{:08b}", value);

    let mut result = Vec::new();
    for c in binary_string.chars() {
        result.push((c as u8) - 48);
    }
    result
}

pub fn binary_to_int(intermediate: &mut [u8]) -> i8 {
    if intermediate[0] == 1 {
        // subtract 1 from intermediate
        let mut index = intermediate.len() - 1;
        while index > 0 {
            if intermediate[index] == 1 {
                intermediate[index] = 0;
                break;
            } else {
                intermediate[index] = 1;
            }
            index -= 1;
        }

        // build complement
        index = 0;
        while index < intermediate.len() {
            if intermediate[index] == 0 {
                intermediate[index] = 1;
            } else {
                intermediate[index] = 0;
            }
            index += 1;
        }

        // calculate binary to decimal
        let mut value = 0;
        for (index, digit) in intermediate.iter().rev().enumerate() {
            value += digit * u8::pow(2, u32::try_from(index).unwrap());
        }
        -(value as i8)
    } else {
        let mut value = 0;
        for (index, digit) in intermediate.iter().rev().enumerate() {
            value += digit * u8::pow(2, u32::try_from(index).unwrap());
        }
        value as i8
    }
}

#[cfg(test)]
mod tests {
    use super::InstructionRegister;
    use crate::assembler::parser::InstructionArgument;

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
    fn test_register_decoding() {
        assert!(matches!(
            InstructionArgument::decode(&[1, 1, 1]),
            InstructionRegister::A
        ));
        assert!(matches!(
            InstructionArgument::decode(&[0, 0, 0]),
            InstructionRegister::B
        ));
        assert!(matches!(
            InstructionArgument::decode(&[0, 0, 1]),
            InstructionRegister::C
        ));
        assert!(matches!(
            InstructionArgument::decode(&[0, 1, 0]),
            InstructionRegister::D
        ));
        assert!(matches!(
            InstructionArgument::decode(&[0, 1, 1]),
            InstructionRegister::E
        ));
        assert!(matches!(
            InstructionArgument::decode(&[1, 0, 0]),
            InstructionRegister::H
        ));
        assert!(matches!(
            InstructionArgument::decode(&[1, 0, 1]),
            InstructionRegister::L
        ));
        assert!(matches!(
            InstructionArgument::decode(&[1, 1, 0]),
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
    fn test_from_index() {
        assert!(matches!(InstructionRegister::from_index(0), InstructionRegister::A));
        assert!(matches!(InstructionRegister::from_index(1), InstructionRegister::B));
        assert!(matches!(InstructionRegister::from_index(2), InstructionRegister::C));
        assert!(matches!(InstructionRegister::from_index(3), InstructionRegister::D));
        assert!(matches!(InstructionRegister::from_index(4), InstructionRegister::E));
        assert!(matches!(InstructionRegister::from_index(5), InstructionRegister::H));
        assert!(matches!(InstructionRegister::from_index(6), InstructionRegister::L));
        assert!(matches!(InstructionRegister::from_index(7), InstructionRegister::M));
    }
}
