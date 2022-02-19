pub use crate::assembler::parser::{
    Instruction, InstructionArgument, InstructionCommand, InstructionRegister,
    InstructionRegisterPair,
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
        let mut file = File::create(&self.output_bin).unwrap();
        for instruction in instructions {
            let encoding = &instruction.encode();
            file.write_all(encoding).unwrap();
        }
    }

    pub fn disassemble(&self, input_bin: String) -> Vec<Instruction> {
        let mut file = File::open(input_bin.to_owned()).unwrap();
        let mut binary_data = Vec::new();

        file.read_to_end(&mut binary_data).unwrap();
        std::fs::remove_file(input_bin).unwrap();

        if binary_data.len() % 8 != 0 {
            panic!("Data is not proper length!");
        }

        let mut raw_instructions = Vec::new();
        for chunk in binary_data.chunks(8) {
            raw_instructions.push(chunk.to_vec());
        }

        self.parse_binary_instructions(&raw_instructions)
    }

    fn parse_binary_instructions(&self, raw_instructions: &[Vec<u8>]) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        let mut index = 0;
        while index < raw_instructions.len() {
            // pretty ugly, maybe there is a better solution with match or something

            let instruction: Instruction;
            // instructions that take up more than one byte (intermediates)
            // MVI
            if raw_instructions[index][0..2] == [0, 0] && raw_instructions[index][5..] == [1, 1, 0]
            {
                let register = InstructionRegister::decode(&raw_instructions[index][2..5]);
                let intermediate = parser::binary_to_int(&raw_instructions[index + 1].to_vec());
                instruction = Instruction::IntermediateRegister(
                    InstructionCommand::Mvi,
                    intermediate,
                    register,
                );
            // LXI
            } else if raw_instructions[index][0..2] == [0, 0]
                && raw_instructions[index][4..] == [0, 0, 0, 1]
            {
                let register_pair = InstructionRegisterPair::decode(&raw_instructions[index][2..4]);

                let intermediate0 =
                    (parser::binary_to_int(&raw_instructions[index + 1].to_vec()) as i16) << 8;
                let intermediate1 =
                    parser::binary_to_int(&raw_instructions[index + 2].to_vec()) as i16;
                instruction = Instruction::Intermediate16Bit(
                    InstructionCommand::Lxi,
                    register_pair,
                    intermediate0 + intermediate1,
                );
            // ADI
            } else if raw_instructions[index] == vec![1, 1, 0, 0, 0, 1, 1, 0] {
                let intermediate = parser::binary_to_int(&raw_instructions[index + 1].to_vec());
                instruction = Instruction::Intermediate(InstructionCommand::Adi, intermediate);
            // ACI
            } else if raw_instructions[index] == vec![1, 1, 0, 0, 1, 1, 1, 0] {
                let intermediate = parser::binary_to_int(&raw_instructions[index + 1].to_vec());
                instruction = Instruction::Intermediate(InstructionCommand::Aci, intermediate);
            // SUI
            } else if raw_instructions[index] == vec![1, 1, 0, 1, 0, 1, 1, 0] {
                let intermediate = parser::binary_to_int(&raw_instructions[index + 1].to_vec());
                instruction = Instruction::Intermediate(InstructionCommand::Sui, intermediate);

            // ORI
            } else if raw_instructions[index] == vec![1, 1, 1, 1, 0, 1, 1, 0] {
                let intermediate = parser::binary_to_int(&raw_instructions[index + 1].to_vec());
                instruction = Instruction::Intermediate(InstructionCommand::Ori, intermediate);

            // XRI
            } else if raw_instructions[index] == vec![1, 1, 1, 0, 1, 1, 1, 0] {
                let intermediate = parser::binary_to_int(&raw_instructions[index + 1].to_vec());
                instruction = Instruction::Intermediate(InstructionCommand::Xri, intermediate);

            // ANI
            } else if raw_instructions[index] == vec![1, 1, 1, 0, 0, 1, 1, 0] {
                let intermediate = parser::binary_to_int(&raw_instructions[index + 1].to_vec());
                instruction = Instruction::Intermediate(InstructionCommand::Ani, intermediate);

            // CPI
            } else if raw_instructions[index] == vec![1, 1, 1, 1, 1, 1, 1, 0] {
                let intermediate = parser::binary_to_int(&raw_instructions[index + 1].to_vec());
                instruction = Instruction::Intermediate(InstructionCommand::Cpi, intermediate);

            // SBI
            } else if raw_instructions[index] == vec![1, 1, 0, 1, 1, 1, 1, 0] {
                let intermediate = parser::binary_to_int(&raw_instructions[index + 1].to_vec());
                instruction = Instruction::Intermediate(InstructionCommand::Sbi, intermediate);

            // instructions without registers
            // HLT
            } else if raw_instructions[index] == vec![0, 1, 1, 1, 0, 1, 1, 0] {
                instruction = Instruction::NoRegister(InstructionCommand::Hlt);

            // STC
            } else if raw_instructions[index] == vec![0, 0, 1, 1, 0, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Stc);

            // CMC
            } else if raw_instructions[index] == vec![0, 0, 1, 1, 1, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Cmc);

            // CMA
            } else if raw_instructions[index] == vec![0, 0, 1, 0, 1, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Cma);

            // RLC
            } else if raw_instructions[index] == vec![0, 0, 0, 0, 0, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Rlc);

            // RRC
            } else if raw_instructions[index] == vec![0, 0, 0, 0, 1, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Rrc);

            // RAL
            } else if raw_instructions[index] == vec![0, 0, 0, 1, 0, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Ral);

            // RAR
            } else if raw_instructions[index] == vec![0, 0, 0, 1, 1, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Rar);

            // DAA
            } else if raw_instructions[index] == vec![0, 0, 1, 0, 0, 1, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Daa);

            // XCHG
            } else if raw_instructions[index] == vec![1, 1, 1, 0, 1, 0, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Xchg);

            // SPHL
            } else if raw_instructions[index] == vec![1, 1, 1, 1, 1, 0, 0, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Sphl);

            // XTHL
            } else if raw_instructions[index] == vec![1, 1, 1, 0, 0, 0, 1, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Xthl);

            // PCHL
            } else if raw_instructions[index] == vec![1, 1, 1, 0, 1, 0, 0, 1] {
                instruction = Instruction::NoRegister(InstructionCommand::Pchl);

            // STA
            } else if raw_instructions[index] == vec![0, 0, 1, 1, 0, 0, 1, 0] {
                let intermediate0 =
                    (parser::binary_to_int(&raw_instructions[index + 1].to_vec()) as i16) << 8;
                let intermediate1 =
                    (parser::binary_to_int(&raw_instructions[index + 2].to_vec()) as i16) & 255;
                instruction = Instruction::Intermediate16BitNoReg(
                    InstructionCommand::Sta,
                    intermediate0 + intermediate1,
                )

            // LDA
            } else if raw_instructions[index] == vec![0, 0, 1, 1, 1, 0, 1, 0] {
                let intermediate0 =
                    (parser::binary_to_int(&raw_instructions[index + 1].to_vec()) as i16) << 8;
                let intermediate1 =
                    (parser::binary_to_int(&raw_instructions[index + 2].to_vec()) as i16) & 255;
                instruction = Instruction::Intermediate16BitNoReg(
                    InstructionCommand::Lda,
                    intermediate0 + intermediate1,
                )

            // SHLD
            } else if raw_instructions[index] == vec![0, 0, 1, 0, 0, 0, 1, 0] {
                let intermediate0 =
                    (parser::binary_to_int(&raw_instructions[index + 1].to_vec()) as i16) << 8;
                let intermediate1 =
                    (parser::binary_to_int(&raw_instructions[index + 2].to_vec()) as i16) & 255;
                instruction = Instruction::Intermediate16BitNoReg(
                    InstructionCommand::Shld,
                    intermediate0 + intermediate1,
                )

            // LHLD
            } else if raw_instructions[index] == vec![0, 0, 1, 0, 1, 0, 1, 0] {
                let intermediate0 =
                    (parser::binary_to_int(&raw_instructions[index + 1].to_vec()) as i16) << 8;
                let intermediate1 =
                    (parser::binary_to_int(&raw_instructions[index + 2].to_vec()) as i16) & 255;
                instruction = Instruction::Intermediate16BitNoReg(
                    InstructionCommand::Lhld,
                    intermediate0 + intermediate1,
                )

            // instructions with 1 argument in the end
            // ADD
            } else if raw_instructions[index][0..5] == [1, 0, 0, 0, 0] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Add, register);

            // ADC
            } else if raw_instructions[index][0..5] == [1, 0, 0, 0, 1] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Adc, register);

            // SUB
            } else if raw_instructions[index][0..5] == [1, 0, 0, 1, 0] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Sub, register);

            // ANA
            } else if raw_instructions[index][0..5] == [1, 0, 1, 0, 0] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Ana, register);

            // ORA
            } else if raw_instructions[index][0..5] == [1, 0, 1, 1, 0] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Ora, register);

            // CMP
            } else if raw_instructions[index][0..5] == [1, 0, 1, 1, 1] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Cmp, register);

            // XRA
            } else if raw_instructions[index][0..5] == [1, 0, 1, 0, 1] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Xra, register);

            // SBB
            } else if raw_instructions[index][0..5] == [1, 0, 0, 1, 1] {
                let register = InstructionRegister::decode(&raw_instructions[index][5..]);
                instruction = Instruction::SingleRegister(InstructionCommand::Sbb, register);

            // instructions with 1 argument in the middle
            // instructions with a register pair
            // STAX
            } else if raw_instructions[index][0..2] == [0, 0]
                && raw_instructions[index][4..] == [0, 0, 1, 0]
            {
                let register_pair = InstructionRegisterPair::decode(&raw_instructions[index][2..4]);
                if matches!(register_pair, InstructionRegisterPair::HL)
                    | matches!(register_pair, InstructionRegisterPair::SP)
                {
                    panic!("cannot use SP or HL in this instruction");
                }

                instruction = Instruction::PairRegister(InstructionCommand::Stax, register_pair);

            // LDAX
            } else if raw_instructions[index][0..2] == [0, 0]
                && raw_instructions[index][4..] == [1, 0, 1, 0]
            {
                let register_pair = InstructionRegisterPair::decode(&raw_instructions[index][2..4]);

                if matches!(register_pair, InstructionRegisterPair::HL)
                    | matches!(register_pair, InstructionRegisterPair::SP)
                {
                    panic!("cannot use SP or HL in this instruction");
                }

                instruction = Instruction::PairRegister(InstructionCommand::Ldax, register_pair);

            // DCX
            } else if raw_instructions[index][0..2] == [0, 0]
                && raw_instructions[index][4..] == [1, 0, 1, 1]
            {
                let register_pair = InstructionRegisterPair::decode(&raw_instructions[index][2..4]);

                instruction = Instruction::PairRegister(InstructionCommand::Dcx, register_pair);

            // INX
            } else if raw_instructions[index][0..2] == [0, 0]
                && raw_instructions[index][4..] == [0, 0, 1, 1]
            {
                let register_pair = InstructionRegisterPair::decode(&raw_instructions[index][2..4]);

                instruction = Instruction::PairRegister(InstructionCommand::Inx, register_pair);

            // DAD
            } else if raw_instructions[index][0..2] == [0, 0]
                && raw_instructions[index][4..] == [1, 0, 0, 1]
            {
                let register_pair = InstructionRegisterPair::decode(&raw_instructions[index][2..4]);

                instruction = Instruction::PairRegister(InstructionCommand::Dad, register_pair);

            // PUSH
            } else if raw_instructions[index][0..2] == [1, 1]
                && raw_instructions[index][4..] == [0, 1, 0, 1]
            {
                let register_pair: InstructionRegisterPair;
                if raw_instructions[index][2..4] == [1, 1] {
                    register_pair = InstructionRegisterPair::FA;
                } else {
                    register_pair = InstructionRegisterPair::decode(&raw_instructions[index][2..4]);
                }

                instruction = Instruction::PairRegister(InstructionCommand::Push, register_pair);

            // POP
            } else if raw_instructions[index][0..2] == [1, 1]
                && raw_instructions[index][4..] == [0, 0, 0, 1]
            {
                let register_pair: InstructionRegisterPair;
                if raw_instructions[index][2..4] == [1, 1] {
                    register_pair = InstructionRegisterPair::FA;
                } else {
                    register_pair = InstructionRegisterPair::decode(&raw_instructions[index][2..4]);
                }

                instruction = Instruction::PairRegister(InstructionCommand::Pop, register_pair);

            // instructions with 1 register in the middle
            // INR
            } else if raw_instructions[index][0..2] == [0, 0]
                && raw_instructions[index][5..] == [1, 0, 0]
            {
                let register = InstructionRegister::decode(&raw_instructions[index][2..5]);
                instruction = Instruction::SingleRegister(InstructionCommand::Inr, register);
            // DCR
            } else if raw_instructions[index][0..2] == [0, 0]
                && raw_instructions[index][5..] == [1, 0, 1]
            {
                let register = InstructionRegister::decode(&raw_instructions[index][2..5]);
                instruction = Instruction::SingleRegister(InstructionCommand::Dcr, register);

            // instructions with 2 registers
            // MOV
            } else if raw_instructions[index][0..2] == [0, 1] {
                let registers = (
                    InstructionRegister::decode(&raw_instructions[index][2..5]),
                    InstructionRegister::decode(&raw_instructions[index][5..]),
                );

                instruction = Instruction::DoubleRegister(InstructionCommand::Mov, registers);
            } else {
                panic!("Invalid instruction!");
            }

            // skip next byte since its the intermediate of the instruction that was just parsed
            if matches!(instruction, Instruction::Intermediate(_, _))
                || matches!(instruction, Instruction::IntermediateRegister(_, _, _))
            {
                index += 2;
            } else if matches!(instruction, Instruction::Intermediate16Bit(_, _, _))
                || matches!(instruction, Instruction::Intermediate16BitNoReg(_, _))
            {
                index += 3;
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
    use super::Assembler;
    use crate::assembler::parser::{
        Instruction, InstructionCommand, InstructionRegister, InstructionRegisterPair,
    };
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_new() {
        let assembler = Assembler::new("test.asm".to_owned(), "test_new_binary".to_owned());
        assert_eq!("test.asm", assembler.input_asm);
        assert_eq!("test_new_binary", assembler.output_bin);
    }

    #[test]
    fn test_assemble() {
        let assembler = Assembler::new("test.asm".to_owned(), "test_assemble_binary".to_owned());
        assembler.assemble();

        let mut file = File::open("test_assemble_binary").unwrap();
        let mut binary_data = Vec::new();

        file.read_to_end(&mut binary_data).unwrap();
        std::fs::remove_file("test_assemble_binary").unwrap();

        assert_eq!(binary_data.len() % 8, 0);
        assert_eq!(binary_data.len(), 512);

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
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 0, 1, 0, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 1, 1, 0, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 0, 0, 0, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 0, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 0, 0, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 0, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 1, 0, 1, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 1, 0, 0, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 1, 0, 1, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 0, 1, 1, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 0, 0, 1, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 1, 1, 1, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 1, 1, 1, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 0, 0, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 0, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 0, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 0, 0, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 0, 1, 0, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 0, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 0, 0, 0, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 0, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 0, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 0, 1, 0, 1, 0]);
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 1, 1]);
        assert_eq!(bytes.next().unwrap(), [1, 0, 1, 0, 0, 0, 0, 0]);
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 0, 1, 0, 0, 1]);
        assert_eq!(bytes.next().unwrap(), [0, 1, 1, 1, 0, 1, 1, 0]);
    }

    #[test]
    fn test_disassemble() {
        let assembler = Assembler::new("test.asm".to_owned(), "test_disassemble_binary".to_owned());
        assembler.assemble();

        let instructions = assembler.disassemble("test_disassemble_binary".to_owned());
        assert_eq!(instructions.len(), 45);

        for (i, instruction) in instructions.iter().enumerate() {
            match instruction {
                Instruction::NoRegister(cmd) => match cmd {
                    InstructionCommand::Stc => {
                        assert_eq!(8, i);
                    }
                    InstructionCommand::Cmc => {
                        assert_eq!(9, i);
                    }
                    InstructionCommand::Cma => {
                        assert_eq!(10, i);
                    }
                    InstructionCommand::Rlc => {
                        assert_eq!(14, i);
                    }
                    InstructionCommand::Rrc => {
                        assert_eq!(15, i);
                    }
                    InstructionCommand::Ral => {
                        assert_eq!(16, i);
                    }
                    InstructionCommand::Rar => {
                        assert_eq!(17, i);
                    }
                    InstructionCommand::Daa => {
                        assert_eq!(19, i);
                    }
                    InstructionCommand::Xchg => {
                        assert_eq!(25, i);
                    }
                    InstructionCommand::Sphl => {
                        assert_eq!(26, i);
                    }
                    InstructionCommand::Xthl => {
                        assert_eq!(27, i);
                    }
                    InstructionCommand::Hlt => {
                        assert_eq!(44, i);
                    }
                    InstructionCommand::Pchl => {
                        assert_eq!(43, i);
                    }
                    _ => panic!("invalid instruction"),
                },
                Instruction::SingleRegister(cmd, register) => match cmd {
                    InstructionCommand::Ana => {
                        assert_eq!(2, i);
                        assert!(matches!(register, InstructionRegister::B))
                    }
                    InstructionCommand::Add => {
                        assert_eq!(3, i);
                        assert!(matches!(register, InstructionRegister::A))
                    }
                    InstructionCommand::Sub => {
                        assert_eq!(4, i);
                        assert!(matches!(register, InstructionRegister::A))
                    }
                    InstructionCommand::Inr => {
                        assert_eq!(5, i);
                        assert!(matches!(register, InstructionRegister::A))
                    }
                    InstructionCommand::Dcr => {
                        assert_eq!(6, i);
                        assert!(matches!(register, InstructionRegister::A))
                    }
                    InstructionCommand::Adc => {
                        assert_eq!(11, i);
                        assert!(matches!(register, InstructionRegister::C))
                    }
                    InstructionCommand::Ora => {
                        assert_eq!(18, i);
                        assert!(matches!(register, InstructionRegister::B))
                    }
                    InstructionCommand::Cmp => {
                        assert_eq!(22, i);
                        assert!(matches!(register, InstructionRegister::B))
                    }
                    InstructionCommand::Xra => {
                        assert_eq!(23, i);
                        assert!(matches!(register, InstructionRegister::B))
                    }
                    InstructionCommand::Sbb => {
                        assert_eq!(24, i);
                        assert!(matches!(register, InstructionRegister::B));
                    }
                    _ => panic!("invalid instruction"),
                },
                Instruction::DoubleRegister(cmd, registers) => match cmd {
                    InstructionCommand::Mov => {
                        assert_eq!(1, i);
                        assert!(matches!(registers.0, InstructionRegister::A));
                        assert!(matches!(registers.1, InstructionRegister::B));
                    }
                    _ => panic!("invalid instruction"),
                },
                Instruction::Intermediate(cmd, intermediate) => match cmd {
                    InstructionCommand::Adi => {
                        assert_eq!(7, i);
                        assert_eq!(-103, *intermediate);
                    }
                    InstructionCommand::Aci => {
                        assert_eq!(12, i);
                        assert_eq!(12, *intermediate);
                    }
                    InstructionCommand::Sui => {
                        assert_eq!(13, i);
                        assert_eq!(12, *intermediate);
                    }
                    InstructionCommand::Ori => {
                        assert_eq!(33, i);
                        assert_eq!(15, *intermediate);
                    }
                    InstructionCommand::Xri => {
                        assert_eq!(34, i);
                        assert_eq!(15, *intermediate);
                    }
                    InstructionCommand::Ani => {
                        assert_eq!(35, i);
                        assert_eq!(-128, *intermediate);
                    }
                    InstructionCommand::Cpi => {
                        assert_eq!(36, i);
                        assert_eq!(15, *intermediate);
                    }
                    InstructionCommand::Sbi => {
                        assert_eq!(37, i);
                        assert_eq!(0, *intermediate);
                    }
                    _ => panic!("invalid instruction"),
                },
                Instruction::IntermediateRegister(cmd, intermediate, register) => match cmd {
                    InstructionCommand::Mvi => {
                        assert_eq!(0, i);
                        assert!(matches!(register, InstructionRegister::A));
                        assert_eq!(28, *intermediate);
                    }
                    _ => panic!("invalid instruction"),
                },
                Instruction::Intermediate16Bit(cmd, register_pair, intermediate) => match cmd {
                    InstructionCommand::Lxi => {
                        assert_eq!(38, i);
                        assert!(matches!(register_pair, InstructionRegisterPair::SP));
                        assert_eq!(12345, *intermediate);
                    }
                    _ => panic!("invalid instruction"),
                },
                Instruction::Intermediate16BitNoReg(cmd, intermediate) => match cmd {
                    InstructionCommand::Sta => {
                        assert_eq!(39, i);
                        assert_eq!(42, *intermediate);
                    }
                    InstructionCommand::Lda => {
                        assert_eq!(40, i);
                        assert_eq!(0, *intermediate);
                    }
                    InstructionCommand::Shld => {
                        assert_eq!(41, i);
                        assert_eq!(12345, *intermediate);
                    }
                    InstructionCommand::Lhld => {
                        assert_eq!(42, i);
                        assert_eq!(4000, *intermediate);
                    }
                    _ => panic!("invalid instruction"),
                },
                Instruction::PairRegister(cmd, register_pair) => match cmd {
                    InstructionCommand::Stax => {
                        assert_eq!(20, i);
                        assert!(matches!(register_pair, InstructionRegisterPair::BC));
                    }
                    InstructionCommand::Ldax => {
                        assert_eq!(21, i);
                        assert!(matches!(register_pair, InstructionRegisterPair::DE));
                    }
                    InstructionCommand::Dcx => {
                        assert_eq!(28, i);
                        assert!(matches!(register_pair, InstructionRegisterPair::BC));
                    }
                    InstructionCommand::Inx => {
                        assert_eq!(29, i);
                        assert!(matches!(register_pair, InstructionRegisterPair::SP));
                    }
                    InstructionCommand::Dad => {
                        assert_eq!(30, i);
                        assert!(matches!(register_pair, InstructionRegisterPair::BC));
                    }
                    InstructionCommand::Push => {
                        assert_eq!(31, i);
                        assert!(matches!(register_pair, InstructionRegisterPair::FA));
                    }
                    InstructionCommand::Pop => {
                        assert_eq!(32, i);
                        assert!(matches!(register_pair, InstructionRegisterPair::FA));
                    }
                    _ => panic!("invalid instruction"),
                },
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_if_corrupted_binary_file() {
        let assembler = Assembler::new(
            "test.asm".to_owned(),
            "data/test/corrupted_binary_file".to_owned(),
        );
        assembler.disassemble("data/test/corrupted_binary_file".to_string());
    }

    #[test]
    #[should_panic]
    fn test_if_unknown_instruction() {
        let assembler = Assembler::new(
            "test.asm".to_owned(),
            "test_if_unknown_instruction_binary".to_owned(),
        );
        let instruction = vec![vec![0, 0, 0, 0, 0, 0, 0, 1]];

        assembler.parse_binary_instructions(&instruction);
    }

    // test ldax and sdax separately since only one register pair is tested
    // in test_disassemble()
    #[test]
    fn test_stax_parsing() {
        let assembler =
            Assembler::new("test.asm".to_owned(), "test_stax_parsing_binary".to_owned());
        let instruction = vec![vec![0, 0, 0, 0, 0, 0, 1, 0]];

        let instruction = &assembler.parse_binary_instructions(&instruction)[0];

        if let Instruction::PairRegister(command, register_pair) = instruction {
            let registers = register_pair.get_registers();
            assert!(matches!(command, InstructionCommand::Stax));
            assert!(matches!(registers.0, InstructionRegister::B));
            assert!(matches!(registers.1, InstructionRegister::C));
        }

        let instruction = vec![vec![0, 0, 0, 1, 0, 0, 1, 0]];

        let instruction = &assembler.parse_binary_instructions(&instruction)[0];

        if let Instruction::PairRegister(command, register_pair) = instruction {
            let registers = register_pair.get_registers();
            assert!(matches!(command, InstructionCommand::Stax));
            assert!(matches!(registers.0, InstructionRegister::D));
            assert!(matches!(registers.1, InstructionRegister::E));
        }
    }

    #[test]
    fn test_ldax_parsing() {
        let assembler =
            Assembler::new("test.asm".to_owned(), "test_ldax_parsing_binary".to_owned());
        let instruction = vec![vec![0, 0, 0, 0, 1, 0, 1, 0]];

        let instruction = &assembler.parse_binary_instructions(&instruction)[0];

        if let Instruction::PairRegister(command, register_pair) = instruction {
            let registers = register_pair.get_registers();
            assert!(matches!(command, InstructionCommand::Ldax));
            assert!(matches!(registers.0, InstructionRegister::B));
            assert!(matches!(registers.1, InstructionRegister::C));
        }

        let instruction = vec![vec![0, 0, 0, 1, 1, 0, 1, 0]];

        let instruction = &assembler.parse_binary_instructions(&instruction)[0];

        if let Instruction::PairRegister(command, register_pair) = instruction {
            let registers = register_pair.get_registers();
            assert!(matches!(command, InstructionCommand::Ldax));
            assert!(matches!(registers.0, InstructionRegister::D));
            assert!(matches!(registers.1, InstructionRegister::E));
        }
    }
}
