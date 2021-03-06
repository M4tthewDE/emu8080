use pest::iterators::Pairs;
use pest::Parser;
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use strum_macros::EnumString;

#[derive(Parser)]
#[grammar = "asm.pest"]
pub struct AssemblyParser;

pub fn parse(file_name: String) -> Vec<Instruction> {
    let unparsed_file = fs::read_to_string(file_name).unwrap();
    let assembly = AssemblyParser::parse(Rule::assembly, &unparsed_file)
        .expect("unsuccessful parse")
        .next()
        .unwrap();

    let raw_instructions = assembly.into_inner();

    let labels = parse_labels(raw_instructions.clone());

    let mut instructions = Vec::new();

    for instruction in raw_instructions {
        let rule = instruction.as_rule();

        // ignore comments and end of input
        if !matches!(rule, Rule::comment | Rule::EOI) {
            let mut inner_instruction_pairs = instruction.into_inner();
            let inner_instruction = inner_instruction_pairs.peek().unwrap();

            let mut rule = inner_instruction.as_rule();

            if matches!(rule, Rule::label) {
                inner_instruction_pairs.next();
            }

            let mut pairs = inner_instruction_pairs.peek().unwrap().into_inner();
            let inner_instruction = pairs.peek().unwrap();
            rule = inner_instruction.as_rule();
            let command = InstructionCommand::from_str(inner_instruction.as_str()).unwrap();
            pairs.next();

            match rule {
                Rule::intermediate_reg_command => {
                    let register =
                        InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap();
                    pairs.next();

                    let mut intermediate = Vec::new();
                    for char in pairs.as_str().chars() {
                        if char == '0' {
                            intermediate.push(0);
                        } else {
                            intermediate.push(1);
                        }
                    }

                    let instruction = Instruction::IntermediateRegister(
                        command,
                        binary_to_int(&intermediate),
                        register,
                    );

                    instructions.push(instruction);
                }
                Rule::intermediate_16_bit_command => {
                    let register_pair: InstructionRegisterPair;

                    let unparsed_register = pairs.peek().unwrap().as_str();

                    // TODO make this prettier
                    if unparsed_register == "SP" {
                        register_pair = InstructionRegisterPair::SP;
                    } else if unparsed_register == "PSW" {
                        register_pair = InstructionRegisterPair::FA;
                    } else {
                        match InstructionRegister::from_str(unparsed_register).unwrap() {
                            InstructionRegister::B => register_pair = InstructionRegisterPair::BC,
                            InstructionRegister::D => register_pair = InstructionRegisterPair::DE,
                            InstructionRegister::H => register_pair = InstructionRegisterPair::DE,
                            _ => panic!("invalid register"),
                        }
                    }

                    pairs.next();

                    let mut raw_intermediate = Vec::new();
                    for char in pairs.as_str().chars() {
                        if char == '0' {
                            raw_intermediate.push(0);
                        } else {
                            raw_intermediate.push(1);
                        }
                    }

                    let high_bits = (binary_to_int(&raw_intermediate[0..8]) as i16) << 8;
                    let low_bits = binary_to_int(&raw_intermediate[8..16]) as i16;

                    let instruction = Instruction::Intermediate16Bit(
                        command,
                        register_pair,
                        high_bits + low_bits,
                    );
                    instructions.push(instruction);
                }
                Rule::double_reg_command => {
                    let register0 =
                        InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap();
                    pairs.next();

                    let register1 =
                        InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap();
                    pairs.next();

                    let instruction = Instruction::DoubleRegister(command, (register0, register1));
                    instructions.push(instruction);
                }
                Rule::single_reg_command => {
                    let register =
                        InstructionRegister::from_str(pairs.peek().unwrap().as_str()).unwrap();
                    pairs.next();

                    let instruction = Instruction::SingleRegister(command, register);
                    instructions.push(instruction);
                }
                Rule::pair_reg_command => {
                    let register_pair: InstructionRegisterPair;

                    let unparsed_register = pairs.peek().unwrap().as_str();

                    // TODO make this prettier
                    if unparsed_register == "SP" {
                        register_pair = InstructionRegisterPair::SP;
                    } else if unparsed_register == "PSW" {
                        register_pair = InstructionRegisterPair::FA;
                    } else {
                        match InstructionRegister::from_str(unparsed_register).unwrap() {
                            InstructionRegister::B => register_pair = InstructionRegisterPair::BC,
                            InstructionRegister::D => register_pair = InstructionRegisterPair::DE,
                            InstructionRegister::H => register_pair = InstructionRegisterPair::DE,
                            _ => panic!("invalid register"),
                        }
                    }

                    pairs.next();

                    let instruction = Instruction::PairRegister(command, register_pair);
                    instructions.push(instruction);
                }
                Rule::intermediate_16_bit_command_no_reg => {
                    let mut raw_intermediate = Vec::new();
                    for char in pairs.as_str().chars() {
                        if char == '0' {
                            raw_intermediate.push(0);
                        } else {
                            raw_intermediate.push(1);
                        }
                    }

                    let high_bits = (binary_to_int(&raw_intermediate[0..8]) as i16) << 8;
                    let low_bits = (binary_to_int(&raw_intermediate[8..16]) as i16) & 255;

                    let instruction =
                        Instruction::Intermediate16BitNoReg(command, high_bits + low_bits);
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

                    let instruction =
                        Instruction::Intermediate(command, binary_to_int(&intermediate));
                    instructions.push(instruction);
                }
                Rule::no_reg_command => {
                    let instruction = Instruction::NoRegister(command);
                    instructions.push(instruction);
                }
                Rule::label_command => {
                    let label = pairs.peek().unwrap().as_span().as_str();
                    let instruction = Instruction::Label(command, *labels.get(label).unwrap());
                    instructions.push(instruction);
                }
                _ => panic!("invalid rule: {:?}", rule),
            }
        }
    }
    instructions
}

fn parse_labels(raw_instructions: Pairs<Rule>) -> HashMap<String, u16> {
    let mut labels = HashMap::new();
    let mut label_address = 0;

    let mut label = "".to_owned();
    for instruction in raw_instructions {
        let rule = instruction.as_rule();

        // ignore comments and end of input
        if !matches!(rule, Rule::comment | Rule::EOI) {
            let mut inner_instruction_pairs = instruction.into_inner();
            let inner_instruction = inner_instruction_pairs.peek().unwrap();

            let mut rule = inner_instruction.as_rule();

            if matches!(rule, Rule::label) {
                label = inner_instruction.as_str().to_string();

                for value in labels.keys() {
                    if label == *value {
                        panic!("can't have duplicate labels: {:?}", label);
                    }
                }

                if InstructionCommand::from_str(&label).is_ok()
                    || InstructionRegister::from_str(&label).is_ok()
                {
                    panic!("label can't occupy reserved names: {:?}", label);
                }

                inner_instruction_pairs.next();
            }

            let mut pairs = inner_instruction_pairs.peek().unwrap().into_inner();
            let inner_instruction = pairs.peek().unwrap();
            rule = inner_instruction.as_rule();
            pairs.next();

            if !label.is_empty() {
                labels.insert(label.trim_end_matches(':').to_owned(), label_address);
                label = "".to_owned();
            }

            match rule {
                Rule::intermediate_reg_command => {
                    label_address += 2;
                }
                Rule::intermediate_16_bit_command => {
                    label_address += 3;
                }
                Rule::double_reg_command => {
                    label_address += 1;
                }
                Rule::single_reg_command => {
                    label_address += 1;
                }
                Rule::pair_reg_command => {
                    label_address += 1;
                }
                Rule::intermediate_16_bit_command_no_reg => {
                    label_address += 3;
                }
                Rule::intermediate_command => {
                    label_address += 2;
                }
                Rule::no_reg_command => {
                    label_address += 1;
                }
                Rule::label_command => {
                    label_address += 3;
                }
                _ => panic!("invalid rule: {:?}", rule),
            }
        }
    }

    labels
}

#[derive(Debug, EnumString, Clone, PartialEq)]
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
    #[strum(serialize = "DCX")]
    Dcx,
    #[strum(serialize = "INX")]
    Inx,
    #[strum(serialize = "DAD")]
    Dad,
    #[strum(serialize = "PUSH")]
    Push,
    #[strum(serialize = "POP")]
    Pop,
    #[strum(serialize = "ORI")]
    Ori,
    #[strum(serialize = "XRI")]
    Xri,
    #[strum(serialize = "ANI")]
    Ani,
    #[strum(serialize = "CPI")]
    Cpi,
    #[strum(serialize = "SBI")]
    Sbi,
    #[strum(serialize = "LXI")]
    Lxi,
    #[strum(serialize = "STA")]
    Sta,
    #[strum(serialize = "LDA")]
    Lda,
    #[strum(serialize = "SHLD")]
    Shld,
    #[strum(serialize = "LHLD")]
    Lhld,
    #[strum(serialize = "PCHL")]
    Pchl,
    #[strum(serialize = "JMP")]
    Jmp,
    #[strum(serialize = "JC")]
    Jc,
    #[strum(serialize = "JZ")]
    Jz,
    #[strum(serialize = "JNC")]
    Jnc,
    #[strum(serialize = "JNZ")]
    Jnz,
    #[strum(serialize = "JM")]
    Jm,
    #[strum(serialize = "JP")]
    Jp,
    #[strum(serialize = "JPE")]
    Jpe,
    #[strum(serialize = "JPO")]
    Jpo,
    #[strum(serialize = "HLT")]
    Hlt,
}

pub trait InstructionArgument {
    fn encode(&self) -> Vec<u8>;
    fn decode(raw_bits: &[u8]) -> Self;
}

#[derive(Debug, Copy, Clone, EnumString, PartialEq)]
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
    fn encode(&self) -> Vec<u8> {
        match self {
            InstructionRegister::A => vec![1, 1, 1],
            InstructionRegister::B => vec![0, 0, 0],
            InstructionRegister::C => vec![0, 0, 1],
            InstructionRegister::D => vec![0, 1, 0],
            InstructionRegister::E => vec![0, 1, 1],
            InstructionRegister::H => vec![1, 0, 0],
            InstructionRegister::L => vec![1, 0, 1],
            InstructionRegister::M => vec![1, 1, 0],
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

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionRegisterPair {
    BC,
    DE,
    HL,
    SP,
    FA,
}

impl InstructionArgument for InstructionRegisterPair {
    fn encode(&self) -> Vec<u8> {
        match self {
            InstructionRegisterPair::BC => vec![0, 0],
            InstructionRegisterPair::DE => vec![0, 1],
            InstructionRegisterPair::HL => vec![1, 0],
            InstructionRegisterPair::SP => vec![1, 1],
            InstructionRegisterPair::FA => vec![1, 1],
        }
    }

    fn decode(raw_bits: &[u8]) -> InstructionRegisterPair {
        match *raw_bits {
            [0, 0] => InstructionRegisterPair::BC,
            [0, 1] => InstructionRegisterPair::DE,
            [1, 0] => InstructionRegisterPair::HL,
            [1, 1] => InstructionRegisterPair::SP,
            _ => panic!("Invalid registerpair"),
        }
    }
}

impl InstructionRegisterPair {
    pub fn get_registers(&self) -> (InstructionRegister, InstructionRegister) {
        match self {
            InstructionRegisterPair::BC => (InstructionRegister::B, InstructionRegister::C),
            InstructionRegisterPair::DE => (InstructionRegister::D, InstructionRegister::E),
            InstructionRegisterPair::HL => (InstructionRegister::H, InstructionRegister::L),
            _ => panic!("invalid register pair"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    NoRegister(InstructionCommand),
    SingleRegister(InstructionCommand, InstructionRegister),
    DoubleRegister(
        InstructionCommand,
        (InstructionRegister, InstructionRegister),
    ),
    Intermediate(InstructionCommand, i8),
    Intermediate16Bit(InstructionCommand, InstructionRegisterPair, i16),
    Intermediate16BitNoReg(InstructionCommand, i16),
    IntermediateRegister(InstructionCommand, i8, InstructionRegister),
    PairRegister(InstructionCommand, InstructionRegisterPair),
    Label(InstructionCommand, u16),
}

impl Instruction {
    pub fn get_size(&self) -> u8 {
        match self {
            Instruction::NoRegister(_) => 1,
            Instruction::SingleRegister(_, _) => 1,
            Instruction::DoubleRegister(_, _) => 1,
            Instruction::Intermediate(_, _) => 2,
            Instruction::Intermediate16Bit(_, _, _) => 3,
            Instruction::Intermediate16BitNoReg(_, _) => 3,
            Instruction::IntermediateRegister(_, _, _) => 2,
            Instruction::PairRegister(_, _) => 1,
            Instruction::Label(_, _) => 3,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        match self {
            Instruction::NoRegister(command) => match command {
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
                InstructionCommand::Daa => {
                    vec![0, 0, 1, 0, 0, 1, 1, 1]
                }
                InstructionCommand::Xchg => {
                    vec![1, 1, 1, 0, 1, 0, 1, 1]
                }
                InstructionCommand::Sphl => {
                    vec![1, 1, 1, 1, 1, 0, 0, 1]
                }
                InstructionCommand::Xthl => {
                    vec![1, 1, 1, 0, 0, 0, 1, 1]
                }
                InstructionCommand::Pchl => {
                    vec![1, 1, 1, 0, 1, 0, 0, 1]
                }
                InstructionCommand::Hlt => {
                    vec![0, 1, 1, 1, 0, 1, 1, 0]
                }
                _ => panic!("invalid instruction"),
            },

            Instruction::SingleRegister(command, register) => match command {
                InstructionCommand::Add => {
                    let mut base_result = vec![1, 0, 0, 0, 0];
                    base_result.append(&mut register.encode());

                    base_result
                }
                InstructionCommand::Adc => {
                    let mut base_result = vec![1, 0, 0, 0, 1];
                    base_result.append(&mut register.encode());

                    base_result
                }
                InstructionCommand::Sub => {
                    let mut base_result = vec![1, 0, 0, 1, 0];
                    base_result.append(&mut register.encode());

                    base_result
                }
                InstructionCommand::Inr => {
                    let mut base_result = vec![0, 0];
                    base_result.append(&mut register.encode());
                    base_result.append(&mut vec![1, 0, 0]);

                    base_result
                }
                InstructionCommand::Dcr => {
                    let mut base_result = vec![0, 0];
                    base_result.append(&mut register.encode());
                    base_result.append(&mut vec![1, 0, 1]);

                    base_result
                }
                InstructionCommand::Ana => {
                    let mut base_result = vec![1, 0, 1, 0, 0];
                    base_result.append(&mut register.encode());

                    base_result
                }
                InstructionCommand::Ora => {
                    let mut base_result = vec![1, 0, 1, 1, 0];
                    base_result.append(&mut register.encode());

                    base_result
                }
                InstructionCommand::Cmp => {
                    let mut base_result = vec![1, 0, 1, 1, 1];
                    base_result.append(&mut register.encode());

                    base_result
                }
                InstructionCommand::Xra => {
                    let mut base_result = vec![1, 0, 1, 0, 1];
                    base_result.append(&mut register.encode());

                    base_result
                }
                InstructionCommand::Sbb => {
                    let mut base_result = vec![1, 0, 0, 1, 1];
                    base_result.append(&mut register.encode());

                    base_result
                }
                _ => panic!("invalid instruction"),
            },

            Instruction::DoubleRegister(command, registers) => match command {
                InstructionCommand::Mov => {
                    let mut base_result = vec![0, 1];
                    base_result.append(&mut registers.0.encode());
                    base_result.append(&mut registers.1.encode());

                    base_result
                }
                _ => panic!("invalid instruction"),
            },

            Instruction::Intermediate(command, intermediate) => match command {
                InstructionCommand::Adi => {
                    let mut base_result = vec![1, 1, 0, 0, 0, 1, 1, 0];
                    base_result.append(&mut int_to_binary(*intermediate as i16, 8));

                    base_result
                }
                InstructionCommand::Aci => {
                    let mut base_result = vec![1, 1, 0, 0, 1, 1, 1, 0];
                    base_result.append(&mut int_to_binary(*intermediate as i16, 8));

                    base_result
                }
                InstructionCommand::Sui => {
                    let mut base_result = vec![1, 1, 0, 1, 0, 1, 1, 0];
                    base_result.append(&mut int_to_binary(*intermediate as i16, 8));

                    base_result
                }
                InstructionCommand::Ori => {
                    let mut base_result = vec![1, 1, 1, 1, 0, 1, 1, 0];
                    base_result.append(&mut int_to_binary(*intermediate as i16, 8));

                    base_result
                }
                InstructionCommand::Xri => {
                    let mut base_result = vec![1, 1, 1, 0, 1, 1, 1, 0];
                    base_result.append(&mut int_to_binary(*intermediate as i16, 8));

                    base_result
                }
                InstructionCommand::Ani => {
                    let mut base_result = vec![1, 1, 1, 0, 0, 1, 1, 0];
                    base_result.append(&mut int_to_binary(*intermediate as i16, 8));

                    base_result
                }
                InstructionCommand::Cpi => {
                    let mut base_result = vec![1, 1, 1, 1, 1, 1, 1, 0];
                    base_result.append(&mut int_to_binary(*intermediate as i16, 8));

                    base_result
                }
                InstructionCommand::Sbi => {
                    let mut base_result = vec![1, 1, 0, 1, 1, 1, 1, 0];
                    base_result.append(&mut int_to_binary(*intermediate as i16, 8));

                    base_result
                }
                _ => panic!("invalid instruction"),
            },

            Instruction::Intermediate16Bit(command, register_pair, intermediate) => match command {
                InstructionCommand::Lxi => {
                    let mut base_result = vec![0, 0];
                    base_result.append(&mut register_pair.encode());
                    base_result.append(&mut vec![0, 0, 0, 1]);
                    base_result.append(&mut int_to_binary(*intermediate, 16));

                    base_result
                }
                _ => panic!("invalid instruction"),
            },

            Instruction::Intermediate16BitNoReg(command, intermediate) => match command {
                InstructionCommand::Sta => {
                    let mut base_result = vec![0, 0, 1, 1, 0, 0, 1, 0];
                    base_result.append(&mut int_to_binary(*intermediate, 16));

                    base_result
                }
                InstructionCommand::Lda => {
                    let mut base_result = vec![0, 0, 1, 1, 1, 0, 1, 0];
                    base_result.append(&mut int_to_binary(*intermediate, 16));

                    base_result
                }
                InstructionCommand::Shld => {
                    let mut base_result = vec![0, 0, 1, 0, 0, 0, 1, 0];
                    base_result.append(&mut int_to_binary(*intermediate, 16));

                    base_result
                }
                InstructionCommand::Lhld => {
                    let mut base_result = vec![0, 0, 1, 0, 1, 0, 1, 0];
                    base_result.append(&mut int_to_binary(*intermediate, 16));

                    base_result
                }
                _ => panic!("invalid instruction"),
            },

            Instruction::IntermediateRegister(command, intermediate, register) => match command {
                InstructionCommand::Mvi => {
                    let mut base_result = vec![0, 0];
                    base_result.append(&mut register.encode());
                    base_result.append(&mut vec![1, 1, 0]);
                    base_result.append(&mut int_to_binary(*intermediate as i16, 8));

                    base_result
                }
                _ => panic!("invalid instruction"),
            },

            Instruction::Label(command, address) => {
                let mut base_result = vec![];
                match command {
                    InstructionCommand::Jmp => {
                        base_result.append(&mut vec![1, 1, 0, 0, 0, 0, 1, 1]);
                        base_result.append(&mut int_to_binary(*address as i16, 16));

                        base_result
                    }
                    InstructionCommand::Jc => {
                        base_result.append(&mut vec![1, 1, 0, 1, 1, 0, 1, 0]);
                        base_result.append(&mut int_to_binary(*address as i16, 16));

                        base_result
                    }
                    InstructionCommand::Jnc => {
                        base_result.append(&mut vec![1, 1, 0, 1, 0, 0, 1, 0]);
                        base_result.append(&mut int_to_binary(*address as i16, 16));

                        base_result
                    }
                    InstructionCommand::Jz => {
                        base_result.append(&mut vec![1, 1, 0, 0, 1, 0, 1, 0]);
                        base_result.append(&mut int_to_binary(*address as i16, 16));

                        base_result
                    }
                    InstructionCommand::Jnz => {
                        base_result.append(&mut vec![1, 1, 0, 0, 0, 0, 1, 0]);
                        base_result.append(&mut int_to_binary(*address as i16, 16));

                        base_result
                    }
                    InstructionCommand::Jm => {
                        base_result.append(&mut vec![1, 1, 1, 1, 1, 0, 1, 0]);
                        base_result.append(&mut int_to_binary(*address as i16, 16));

                        base_result
                    }
                    InstructionCommand::Jp => {
                        base_result.append(&mut vec![1, 1, 1, 1, 0, 0, 1, 0]);
                        base_result.append(&mut int_to_binary(*address as i16, 16));

                        base_result
                    }
                    InstructionCommand::Jpe => {
                        base_result.append(&mut vec![1, 1, 1, 0, 1, 0, 1, 0]);
                        base_result.append(&mut int_to_binary(*address as i16, 16));

                        base_result
                    }
                    InstructionCommand::Jpo => {
                        base_result.append(&mut vec![1, 1, 1, 0, 0, 0, 1, 0]);
                        base_result.append(&mut int_to_binary(*address as i16, 16));

                        base_result
                    }
                    _ => panic!("invalid instruction"),
                }
            }

            Instruction::PairRegister(command, register_pair) => {
                let mut base_result = vec![0, 0];
                match command {
                    InstructionCommand::Stax => {
                        base_result.append(&mut register_pair.encode());
                        base_result.append(&mut vec![0, 0, 1, 0]);

                        base_result
                    }
                    InstructionCommand::Ldax => {
                        base_result.append(&mut register_pair.encode());
                        base_result.append(&mut vec![1, 0, 1, 0]);

                        base_result
                    }
                    InstructionCommand::Dcx => {
                        base_result.append(&mut register_pair.encode());
                        base_result.append(&mut vec![1, 0, 1, 1]);

                        base_result
                    }
                    InstructionCommand::Inx => {
                        base_result.append(&mut register_pair.encode());
                        base_result.append(&mut vec![0, 0, 1, 1]);

                        base_result
                    }
                    InstructionCommand::Dad => {
                        base_result.append(&mut register_pair.encode());
                        base_result.append(&mut vec![1, 0, 0, 1]);

                        base_result
                    }
                    InstructionCommand::Push => {
                        base_result = vec![1, 1];
                        if matches!(register_pair, InstructionRegisterPair::SP) {
                            panic!("can not use SP in this instruction");
                        }
                        base_result.append(&mut register_pair.encode());
                        base_result.append(&mut vec![0, 1, 0, 1]);

                        base_result
                    }
                    InstructionCommand::Pop => {
                        base_result = vec![1, 1];
                        if matches!(register_pair, InstructionRegisterPair::SP) {
                            panic!("can not use SP in this instruction");
                        }
                        base_result.append(&mut register_pair.encode());
                        base_result.append(&mut vec![0, 0, 0, 1]);

                        base_result
                    }
                    _ => panic!("invalid instruction"),
                }
            }
        }
    }
}

pub fn int_to_binary(value: i16, mut size: i8) -> Vec<u8> {
    let mut result = Vec::new();
    size -= 1;
    while size >= 0 {
        let bit = (value >> size) & 1;
        result.push(bit as u8);
        size -= 1;
    }

    result
}

pub fn binary_to_int(intermediate: &[u8]) -> i8 {
    let mut result = 0;

    for (i, num) in intermediate.iter().enumerate() {
        result |= num;

        if i != 7 {
            result <<= 1;
        }
    }

    result as i8
}

#[cfg(test)]
mod tests {
    use crate::assembler::parser::binary_to_int;
    use crate::assembler::parser::int_to_binary;

    use super::parse;
    use super::{InstructionArgument, InstructionRegister, InstructionRegisterPair};

    #[test]
    fn test_register_encoding() {
        assert_eq!(*InstructionRegister::A.encode(), vec![1, 1, 1]);
        assert_eq!(*InstructionRegister::B.encode(), vec![0, 0, 0]);
        assert_eq!(*InstructionRegister::C.encode(), vec![0, 0, 1]);
        assert_eq!(*InstructionRegister::D.encode(), vec![0, 1, 0]);
        assert_eq!(*InstructionRegister::E.encode(), vec![0, 1, 1]);
        assert_eq!(*InstructionRegister::H.encode(), vec![1, 0, 0]);
        assert_eq!(*InstructionRegister::L.encode(), vec![1, 0, 1]);
        assert_eq!(*InstructionRegister::M.encode(), vec![1, 1, 0]);
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
    fn test_register_pair_encoding() {
        assert_eq!(InstructionRegisterPair::BC.encode(), &[0, 0]);
        assert_eq!(InstructionRegisterPair::DE.encode(), &[0, 1]);
        assert_eq!(InstructionRegisterPair::HL.encode(), &[1, 0]);
        assert_eq!(InstructionRegisterPair::SP.encode(), &[1, 1]);
    }

    #[test]
    fn test_register_pair_decoding() {
        assert!(matches!(
            InstructionRegisterPair::decode(&[0, 0]),
            InstructionRegisterPair::BC
        ));
        assert!(matches!(
            InstructionRegisterPair::decode(&[0, 1]),
            InstructionRegisterPair::DE
        ));
        assert!(matches!(
            InstructionRegisterPair::decode(&[1, 0]),
            InstructionRegisterPair::HL
        ));
        assert!(matches!(
            InstructionRegisterPair::decode(&[1, 1]),
            InstructionRegisterPair::SP
        ));
    }

    #[test]
    #[should_panic]
    fn test_invalid_register_pair_decoding() {
        InstructionRegisterPair::decode(&[1, 1, 1]);
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
        assert!(matches!(
            InstructionRegister::from_index(0),
            InstructionRegister::A
        ));
        assert!(matches!(
            InstructionRegister::from_index(1),
            InstructionRegister::B
        ));
        assert!(matches!(
            InstructionRegister::from_index(2),
            InstructionRegister::C
        ));
        assert!(matches!(
            InstructionRegister::from_index(3),
            InstructionRegister::D
        ));
        assert!(matches!(
            InstructionRegister::from_index(4),
            InstructionRegister::E
        ));
        assert!(matches!(
            InstructionRegister::from_index(5),
            InstructionRegister::H
        ));
        assert!(matches!(
            InstructionRegister::from_index(6),
            InstructionRegister::L
        ));
        assert!(matches!(
            InstructionRegister::from_index(7),
            InstructionRegister::M
        ));
    }

    #[test]
    fn test_binary_to_int() {
        assert_eq!(binary_to_int(&mut vec![0, 0, 0, 0, 1, 1, 1, 1]), 15);
        assert_eq!(binary_to_int(&mut vec![1, 0, 0, 0, 0, 0, 0, 0]), -128);
    }

    #[test]
    fn test_int_to_binary() {
        assert_eq!(int_to_binary(15, 8), vec![0, 0, 0, 0, 1, 1, 1, 1]);
        assert_eq!(int_to_binary(-128, 8), vec![1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            int_to_binary(4000, 16),
            vec![0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    #[should_panic]
    fn test_duplicate_labels() {
        parse("data/test/duplicate_labels.asm".to_string());
    }
}
