use crate::assembler::parser::Encoding;
pub use crate::assembler::parser::{
    Instruction, InstructionCommand, InstructionRegister, InstructionType,
};
use std::fs::File;
use std::io::{Read, Write};

mod parser;

#[derive(Debug)]
pub struct Assembler {
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
        let parse_result = parser::parse(self.input_asm.to_owned());
        let instructions = parse_result.0;

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
                && !matches!(
                    InstructionRegister::decode(&raw_instructions[index][2..5]),
                    InstructionRegister::Invalid
                )
                && raw_instructions[index][5..] == [1, 1, 0]
            {
                instruction = Instruction {
                    variant: InstructionType::IntermediateReg,
                    command: InstructionCommand::Mvi,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][2..5])],
                    intermediate: parser::binary_to_int(&mut raw_instructions[index + 1].to_vec()),
                };
            // ADI
            } else if raw_instructions[index] == [1, 1, 0, 0, 0, 1, 1, 0] {
                instruction = Instruction {
                    variant: InstructionType::Intermediate,
                    command: InstructionCommand::Adi,
                    registers: vec![],
                    intermediate: parser::binary_to_int(&mut raw_instructions[index + 1].to_vec()),
                };
            // ACI
            } else if raw_instructions[index] == [1, 1, 0, 0, 1, 1, 1, 0] {
                instruction = Instruction {
                    variant: InstructionType::Intermediate,
                    command: InstructionCommand::Aci,
                    registers: vec![],
                    intermediate: parser::binary_to_int(&mut raw_instructions[index + 1].to_vec()),
                };
            } else if raw_instructions[index] == [1, 1, 0, 1, 0, 1, 1, 0] {
                instruction = Instruction {
                    variant: InstructionType::Intermediate,
                    command: InstructionCommand::Sui,
                    registers: vec![],
                    intermediate: parser::binary_to_int(&mut raw_instructions[index + 1].to_vec()),
                };
            // instructions without registers
            // HLT
            } else if raw_instructions[index] == [0, 1, 1, 1, 0, 1, 1, 0] {
                instruction = Instruction {
                    variant: InstructionType::NoReg,
                    command: InstructionCommand::Hlt,
                    registers: Vec::new(),
                    intermediate: 0,
                };
            // STC
            } else if raw_instructions[index] == [0, 0, 1, 1, 0, 1, 1, 1] {
                instruction = Instruction {
                    variant: InstructionType::NoReg,
                    command: InstructionCommand::Stc,
                    registers: Vec::new(),
                    intermediate: 0,
                }
            // CMC
            } else if raw_instructions[index] == [0, 0, 1, 1, 1, 1, 1, 1] {
                instruction = Instruction {
                    variant: InstructionType::NoReg,
                    command: InstructionCommand::Cmc,
                    registers: Vec::new(),
                    intermediate: 0,
                }
            // CMA
            } else if raw_instructions[index] == [0, 0, 1, 0, 1, 1, 1, 1] {
                instruction = Instruction {
                    variant: InstructionType::NoReg,
                    command: InstructionCommand::Cma,
                    registers: Vec::new(),
                    intermediate: 0,
                }
            // RLC
            } else if raw_instructions[index] == [0, 0, 0, 0, 0, 1, 1, 1] {
                instruction = Instruction {
                    variant: InstructionType::NoReg,
                    command: InstructionCommand::Rlc,
                    registers: Vec::new(),
                    intermediate: 0,
                }
            // RRC
            } else if raw_instructions[index] == [0, 0, 0, 0, 1, 1, 1, 1] {
                instruction = Instruction {
                    variant: InstructionType::NoReg,
                    command: InstructionCommand::Rrc,
                    registers: Vec::new(),
                    intermediate: 0,
                }
            // RAL
            } else if raw_instructions[index] == [0, 0, 0, 1, 0, 1, 1, 1] {
                instruction = Instruction {
                    variant: InstructionType::NoReg,
                    command: InstructionCommand::Ral,
                    registers: Vec::new(),
                    intermediate: 0,
                }
            // RAR
            } else if raw_instructions[index] == [0, 0, 0, 1, 1, 1, 1, 1] {
                instruction = Instruction {
                    variant: InstructionType::NoReg,
                    command: InstructionCommand::Rar,
                    registers: Vec::new(),
                    intermediate: 0,
                }
            // DAA
            } else if raw_instructions[index] == [0, 0, 1, 0, 0, 1, 1, 1] {
                instruction = Instruction {
                    variant: InstructionType::NoReg,
                    command: InstructionCommand::Daa,
                    registers: Vec::new(),
                    intermediate: 0,
                }
            // instructions with 1 argument in the end
            // ADD
            } else if raw_instructions[index][0..5] == [1, 0, 0, 0, 0]
                && !matches!(
                    InstructionRegister::decode(&raw_instructions[index][5..]),
                    InstructionRegister::Invalid
                )
            {
                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Add,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][5..])],
                    intermediate: 0,
                }
            // ADC
            } else if raw_instructions[index][0..5] == [1, 0, 0, 0, 1]
                && !matches!(
                    InstructionRegister::decode(&raw_instructions[index][5..]),
                    InstructionRegister::Invalid
                )
            {
                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Adc,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][5..])],
                    intermediate: 0,
                }
            // SUB
            } else if raw_instructions[index][0..5] == [1, 0, 0, 1, 0]
                && !matches!(
                    InstructionRegister::decode(&raw_instructions[index][5..]),
                    InstructionRegister::Invalid
                )
            {
                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Sub,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][5..])],
                    intermediate: 0,
                }
            // ANA
            } else if raw_instructions[index][0..5] == [1, 0, 1, 0, 0]
                && !matches!(
                    InstructionRegister::decode(&raw_instructions[index][5..]),
                    InstructionRegister::Invalid
                )
            {
                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Ana,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][5..])],
                    intermediate: 0,
                }
            // ORA
            } else if raw_instructions[index][0..5] == [1, 0, 1, 1, 0]
                && !matches!(
                    InstructionRegister::decode(&raw_instructions[index][5..]),
                    InstructionRegister::Invalid
                )
            {
                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Ora,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][5..])],
                    intermediate: 0,
                }
            // CMP
            } else if raw_instructions[index][0..5] == [1, 0, 1, 1, 1]
                && !matches!(
                    InstructionRegister::decode(&raw_instructions[index][5..]),
                    InstructionRegister::Invalid
                )
            {
                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Cmp,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][5..])],
                    intermediate: 0,
                }
            // XRA
            } else if raw_instructions[index][0..5] == [1, 0, 1, 0, 1]
                && !matches!(
                    InstructionRegister::decode(&raw_instructions[index][5..]),
                    InstructionRegister::Invalid
                )
            {
                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Xra,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][5..])],
                    intermediate: 0,
                }
            // SBB
            } else if raw_instructions[index][0..5] == [1, 0, 0, 1, 1]
                && !matches!(
                    InstructionRegister::decode(&raw_instructions[index][5..]),
                    InstructionRegister::Invalid
                )
            {
                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Sbb,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][5..])],
                    intermediate: 0,
                }
            // instructions with 1 argument in the middle
            // instructions with a register pair
            // STAX
            } else if raw_instructions[index][0..3] == [0, 0, 0]
                && raw_instructions[index][4..] == [0, 0, 1, 0]
            {
                let registers: Vec<InstructionRegister>;
                if raw_instructions[index][4] == 0 {
                    registers = vec![InstructionRegister::B, InstructionRegister::C];
                } else {
                    registers = vec![InstructionRegister::D, InstructionRegister::E];
                }

                instruction = Instruction {
                    variant: InstructionType::PairReg,
                    command: InstructionCommand::Stax,
                    registers,
                    intermediate: 0,
                }
            // LDAX
            } else if raw_instructions[index][0..3] == [0, 0, 0]
                && raw_instructions[index][4..] == [1, 0, 1, 0]
            {
                let registers: Vec<InstructionRegister>;
                if raw_instructions[index][4] == 0 {
                    registers = vec![InstructionRegister::B, InstructionRegister::C];
                } else {
                    registers = vec![InstructionRegister::D, InstructionRegister::E];
                }

                instruction = Instruction {
                    variant: InstructionType::PairReg,
                    command: InstructionCommand::Ldax,
                    registers,
                    intermediate: 0,
                }
            // instructions with 1 register in the middle
            // INR
            } else if raw_instructions[index][0..2] == [0, 0]
                && raw_instructions[index][5..] == [1, 0, 0]
                && !matches!(
                    InstructionRegister::decode(&raw_instructions[index][2..5]),
                    InstructionRegister::Invalid
                )
            {
                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Inr,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][2..5])],
                    intermediate: 0,
                }
            // DCR
            } else if raw_instructions[index][0..2] == [0, 0]
                && raw_instructions[index][5..] == [1, 0, 1]
                && !matches!(
                    InstructionRegister::decode(&raw_instructions[index][2..5]),
                    InstructionRegister::Invalid
                )
            {
                instruction = Instruction {
                    variant: InstructionType::SingleReg,
                    command: InstructionCommand::Dcr,
                    registers: vec![InstructionRegister::decode(&raw_instructions[index][2..5])],
                    intermediate: 0,
                }
            // instructions with 2 registers
            // MOV
            } else if raw_instructions[index][0..2] == [0, 1]
                && !matches!(
                    InstructionRegister::decode(&raw_instructions[index][2..5]),
                    InstructionRegister::Invalid
                )
                && !matches!(
                    InstructionRegister::decode(&raw_instructions[index][5..]),
                    InstructionRegister::Invalid
                )
            {
                let args = vec![
                    InstructionRegister::decode(&raw_instructions[index][2..5]),
                    InstructionRegister::decode(&raw_instructions[index][5..]),
                ];

                instruction = Instruction {
                    variant: InstructionType::DoubleReg,
                    command: InstructionCommand::Mov,
                    registers: args,
                    intermediate: 0,
                }
            } else {
                panic!("Invalid instruction!");
            }

            // skip next byte since its the intermediate of the instruction that was just parsed
            if matches!(instruction.variant, InstructionType::Intermediate)
                || matches!(instruction.variant, InstructionType::IntermediateReg)
            {
                index += 2;
            } else {
                index += 1;
            }
            instructions.push(instruction);
        }
        instructions
    }
}

#[cfg(test)]
mod tests {
    use super::parser::{InstructionCommand, InstructionRegister, InstructionType};
    use super::Assembler;
    use std::fs::File;
    use std::io::Read;

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
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 1, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 1, 1, 1, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 1, 1, 1, 1, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 1, 0, 0, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 0, 0, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 1, 0, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 1, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 1, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 0, 0, 1, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 1, 1, 0, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 0, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 0, 1, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 0, 1, 0, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 0, 1, 1, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 1, 0, 1, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 1, 0, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 1, 1, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 1, 1, 0, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 0, 0, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 1, 1, 0, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 1, 1, 1, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 1, 0, 1, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 1, 1, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 1, 1, 1, 0, 1, 1, 0]);
    }

    #[test]
    fn test_disassemble() {
        let assembler = Assembler::new("test.asm".to_owned(), "output".to_owned());
        assembler.assemble();

        let instructions = assembler.disassemble("output".to_owned());
        assert_eq!(instructions.len(), 26);

        assert!(matches!(
            instructions[0].variant,
            InstructionType::IntermediateReg
        ));
        assert!(matches!(instructions[0].command, InstructionCommand::Mvi));
        assert!(matches!(
            instructions[0].registers[0],
            InstructionRegister::A
        ));
        assert_eq!(instructions[0].intermediate, 28);

        assert!(matches!(
            instructions[1].variant,
            InstructionType::DoubleReg
        ));
        assert!(matches!(instructions[1].command, InstructionCommand::Mov));
        assert!(matches!(
            instructions[1].registers[0],
            InstructionRegister::A
        ));
        assert!(matches!(
            instructions[1].registers[1],
            InstructionRegister::B
        ));

        assert!(matches!(
            instructions[2].variant,
            InstructionType::SingleReg
        ));
        assert!(matches!(instructions[2].command, InstructionCommand::Ana));
        assert!(matches!(
            instructions[2].registers[0],
            InstructionRegister::B
        ));

        assert!(matches!(
            instructions[3].variant,
            InstructionType::SingleReg
        ));
        assert!(matches!(instructions[3].command, InstructionCommand::Add));
        assert!(matches!(
            instructions[3].registers[0],
            InstructionRegister::A
        ));

        assert!(matches!(
            instructions[4].variant,
            InstructionType::SingleReg
        ));
        assert!(matches!(instructions[4].command, InstructionCommand::Sub));
        assert!(matches!(
            instructions[4].registers[0],
            InstructionRegister::A
        ));

        assert!(matches!(
            instructions[5].variant,
            InstructionType::SingleReg
        ));
        assert!(matches!(instructions[5].command, InstructionCommand::Inr));
        assert!(matches!(
            instructions[5].registers[0],
            InstructionRegister::A
        ));

        assert!(matches!(
            instructions[6].variant,
            InstructionType::SingleReg
        ));
        assert!(matches!(instructions[6].command, InstructionCommand::Dcr));
        assert!(matches!(
            instructions[6].registers[0],
            InstructionRegister::A
        ));

        assert!(matches!(
            instructions[7].variant,
            InstructionType::Intermediate
        ));
        assert!(matches!(instructions[7].command, InstructionCommand::Adi));
        assert_eq!(instructions[7].intermediate, -103);

        assert!(matches!(instructions[8].variant, InstructionType::NoReg));
        assert!(matches!(instructions[8].command, InstructionCommand::Stc));

        assert!(matches!(instructions[9].variant, InstructionType::NoReg));
        assert!(matches!(instructions[9].command, InstructionCommand::Cmc));

        assert!(matches!(instructions[10].variant, InstructionType::NoReg));
        assert!(matches!(instructions[10].command, InstructionCommand::Cma));

        assert!(matches!(
            instructions[11].variant,
            InstructionType::SingleReg
        ));
        assert!(matches!(instructions[11].command, InstructionCommand::Adc));
        assert!(matches!(
            instructions[11].registers[0],
            InstructionRegister::C
        ));

        assert!(matches!(
            instructions[12].variant,
            InstructionType::Intermediate
        ));
        assert!(matches!(instructions[12].command, InstructionCommand::Aci));
        assert_eq!(instructions[12].intermediate, 12);

        assert!(matches!(
            instructions[12].variant,
            InstructionType::Intermediate
        ));
        assert!(matches!(instructions[13].command, InstructionCommand::Sui));
        assert_eq!(instructions[13].intermediate, 12);

        assert!(matches!(instructions[14].variant, InstructionType::NoReg));
        assert!(matches!(instructions[14].command, InstructionCommand::Rlc));

        assert!(matches!(instructions[15].variant, InstructionType::NoReg));
        assert!(matches!(instructions[15].command, InstructionCommand::Rrc));

        assert!(matches!(instructions[16].variant, InstructionType::NoReg));
        assert!(matches!(instructions[16].command, InstructionCommand::Ral));

        assert!(matches!(instructions[17].variant, InstructionType::NoReg));
        assert!(matches!(instructions[17].command, InstructionCommand::Rar));

        assert!(matches!(
            instructions[18].variant,
            InstructionType::SingleReg
        ));
        assert!(matches!(
            instructions[18].registers[0],
            InstructionRegister::B
        ));
        assert!(matches!(instructions[18].command, InstructionCommand::Ora));

        assert!(matches!(instructions[19].variant, InstructionType::NoReg));
        assert!(matches!(instructions[19].command, InstructionCommand::Daa));

        assert!(matches!(instructions[20].variant, InstructionType::PairReg));
        assert!(matches!(instructions[20].command, InstructionCommand::Stax));
        assert!(matches!(
            instructions[20].registers[0],
            InstructionRegister::B
        ));
        assert!(matches!(
            instructions[20].registers[1],
            InstructionRegister::C
        ));

        assert!(matches!(instructions[21].variant, InstructionType::PairReg));
        assert!(matches!(instructions[21].command, InstructionCommand::Ldax));
        assert!(matches!(
            instructions[21].registers[0],
            InstructionRegister::D
        ));
        assert!(matches!(
            instructions[21].registers[1],
            InstructionRegister::E
        ));

        assert!(matches!(
            instructions[22].variant,
            InstructionType::SingleReg
        ));
        assert!(matches!(
            instructions[22].registers[0],
            InstructionRegister::B
        ));
        assert!(matches!(instructions[22].command, InstructionCommand::Cmp));

        assert!(matches!(
            instructions[23].variant,
            InstructionType::SingleReg
        ));
        assert!(matches!(
            instructions[23].registers[0],
            InstructionRegister::B
        ));
        assert!(matches!(instructions[23].command, InstructionCommand::Xra));

        assert!(matches!(
            instructions[24].variant,
            InstructionType::SingleReg
        ));
        assert!(matches!(
            instructions[24].registers[0],
            InstructionRegister::B
        ));
        assert!(matches!(instructions[24].command, InstructionCommand::Sbb));

        assert!(matches!(instructions[25].variant, InstructionType::NoReg));
        assert!(matches!(instructions[25].command, InstructionCommand::Hlt));
    }
}
