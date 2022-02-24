pub use crate::assembler::parser::{
    Instruction, InstructionArgument, InstructionCommand, InstructionRegister,
    InstructionRegisterPair,
};
use std::collections::HashMap;
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
        let instructions = parser::parse(self.input_asm.to_owned());

        // write to file
        let mut file = File::create(&self.output_bin).unwrap();
        for instruction in instructions {
            let encoding = &instruction.encode();
            file.write_all(encoding).unwrap();
        }
    }

    pub fn disassemble(&self, input_bin: String) -> HashMap<u16, Instruction> {
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

    fn parse_binary_instructions(&self, raw_instructions: &[Vec<u8>]) -> HashMap<u16, Instruction> {
        let mut instructions = HashMap::new();

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

            // JMP
            } else if raw_instructions[index] == vec![1, 1, 0, 0, 0, 0, 1, 1] {
                let address0 =
                    (parser::binary_to_int(&raw_instructions[index + 1].to_vec()) as u16) << 8;
                let address1 =
                    (parser::binary_to_int(&raw_instructions[index + 2].to_vec()) as u16) & 255;
                instruction = Instruction::Label(InstructionCommand::Jmp, address0 + address1)

            // JC
            } else if raw_instructions[index] == vec![1, 1, 0, 1, 1, 0, 1, 0] {
                let address0 =
                    (parser::binary_to_int(&raw_instructions[index + 1].to_vec()) as u16) << 8;
                let address1 =
                    (parser::binary_to_int(&raw_instructions[index + 2].to_vec()) as u16) & 255;
                instruction = Instruction::Label(InstructionCommand::Jc, address0 + address1)

            // JNC
            } else if raw_instructions[index] == vec![1, 1, 0, 1, 0, 0, 1, 0] {
                let address0 =
                    (parser::binary_to_int(&raw_instructions[index + 1].to_vec()) as u16) << 8;
                let address1 =
                    (parser::binary_to_int(&raw_instructions[index + 2].to_vec()) as u16) & 255;
                instruction = Instruction::Label(InstructionCommand::Jnc, address0 + address1)

            // JZ
            } else if raw_instructions[index] == vec![1, 1, 0, 0, 1, 0, 1, 0] {
                let address0 =
                    (parser::binary_to_int(&raw_instructions[index + 1].to_vec()) as u16) << 8;
                let address1 =
                    (parser::binary_to_int(&raw_instructions[index + 2].to_vec()) as u16) & 255;
                instruction = Instruction::Label(InstructionCommand::Jz, address0 + address1)

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

            instructions.insert(index as u16, instruction.clone());

            // skip next byte since its the intermediate of the instruction that was just parsed
            if matches!(instruction, Instruction::Intermediate(_, _))
                || matches!(instruction, Instruction::IntermediateRegister(_, _, _))
            {
                index += 2;
            } else if matches!(instruction, Instruction::Intermediate16Bit(_, _, _))
                || matches!(instruction, Instruction::Intermediate16BitNoReg(_, _))
                || matches!(instruction, Instruction::Label(_, _))
            {
                index += 3;
            } else {
                index += 1;
            }
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
        let assembler = Assembler::new("data/test/end_to_end.asm".to_owned(), "test_assemble_binary".to_owned());
        assembler.assemble();

        let mut file = File::open("test_assemble_binary").unwrap();
        let mut binary_data = Vec::new();

        file.read_to_end(&mut binary_data).unwrap();
        std::fs::remove_file("test_assemble_binary").unwrap();

        assert_eq!(binary_data.len() % 8, 0);
        assert_eq!(binary_data.len(), 616);

        let mut bytes = binary_data.chunks(8);

        // MVI
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 1, 1, 0]);
        // MVI intermediate
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 1, 1, 1, 0, 0]);
        // MOV
        assert_eq!(bytes.next().unwrap(), [0, 1, 1, 1, 1, 0, 0, 0]);
        // ANA
        assert_eq!(bytes.next().unwrap(), [1, 0, 1, 0, 0, 0, 0, 0]);
        // ADD
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 0, 0, 1, 1, 1]);
        // SUB
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 1, 0, 1, 1, 1]);
        // INR
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 1, 0, 0]);
        // DCR
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 1, 0, 1]);
        // ADI
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 0, 0, 1, 1, 0]);
        // ADI intermediate
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 1, 1, 0, 0, 1]);
        // STC
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 0, 1, 1, 1]);
        // CMC
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 1, 1, 1]);
        // CMA
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 0, 1, 1, 1, 1]);
        // ADC
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 0, 1, 0, 0, 1]);
        // ACI
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 0, 1, 1, 1, 0]);
        // ACI intermediate
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 0, 0]);
        // SUI
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 1, 0, 1, 1, 0]);
        // SUI intermediate
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 0, 0]);
        // RLC
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 1, 1, 1]);
        // RRC
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 1, 1]);
        // RAL
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 1, 0, 1, 1, 1]);
        // RAR
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 1, 1, 1, 1, 1]);
        // ORA
        assert_eq!(bytes.next().unwrap(), [1, 0, 1, 1, 0, 0, 0, 0]);
        // DAA
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 0, 0, 1, 1, 1]);
        // STAX
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 1, 0]);
        // LDAX
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 1, 1, 0, 1, 0]);
        // CMP
        assert_eq!(bytes.next().unwrap(), [1, 0, 1, 1, 1, 0, 0, 0]);
        // XRA
        assert_eq!(bytes.next().unwrap(), [1, 0, 1, 0, 1, 0, 0, 0]);
        // SBB
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 1, 1, 0, 0, 0]);
        // XCHG
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 0, 1, 0, 1, 1]);
        // SPHL
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 1, 1, 0, 0, 1]);
        // XTHL
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 0, 0, 0, 1, 1]);
        // DCX
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 0, 1, 1]);
        // INX
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 0, 0, 1, 1]);
        // DAD
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 0, 0, 1]);
        // PUSH
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 1, 0, 1, 0, 1]);
        // POP
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 1, 0, 0, 0, 1]);
        // ORI
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 1, 0, 1, 1, 0]);
        // ORI intermediate
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 1, 1]);
        // XRI
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 0, 1, 1, 1, 0]);
        // XRI intermediate
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 1, 1]);
        // ANI
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 0, 0, 1, 1, 0]);
        // ANI intermediate
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 0, 0, 0, 0, 0]);
        // CPI
        assert_eq!(bytes.next().unwrap(), [1, 1, 1, 1, 1, 1, 1, 0]);
        // CPI intermediate
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 1, 1]);
        // SBI
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 1, 1, 1, 1, 0]);
        // SBI intermediate
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 0, 0]);
        // LXI
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 0, 0, 0, 1]);
        // LXI intermediate one
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 0, 0, 0, 0]);
        // LXI intermediate two
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 0, 0, 1]);
        // STA
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 0, 0, 1, 0]);
        // STA intermediate one
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 0, 0]);
        // STA intermediate two
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 0, 1, 0, 1, 0]);
        // LDA
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 0, 1, 0]);
        // LDA intermediate one
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 0, 0]);
        // LDA intermediate two
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 0, 0]);
        // SHLD
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 0, 0, 0, 1, 0]);
        // SHLD intermediate one
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 0, 0, 0, 0]);
        // SHLD intermediate two
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 1, 1, 0, 0, 1]);
        // LHLD
        assert_eq!(bytes.next().unwrap(), [0, 0, 1, 0, 1, 0, 1, 0]);
        // LHLD intemediate one
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 1, 1, 1, 1]);
        // LHLD intemediate two
        assert_eq!(bytes.next().unwrap(), [1, 0, 1, 0, 0, 0, 0, 0]);
        // JMP
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 0, 0, 0, 1, 1]);
        // JMP address one
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 0, 0]);
        // JMP address two
        assert_eq!(bytes.next().unwrap(), [0, 1, 0, 0, 1, 0, 1, 1]);
        // JC
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 1, 1, 0, 1, 0]);
        // JC address one
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 0, 0]);
        // JC address two
        assert_eq!(bytes.next().unwrap(), [0, 1, 0, 0, 1, 0, 1, 1]);
        // JNC
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 1, 0, 0, 1, 0]);
        // JNC address one
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 0, 0]);
        // JNC address two
        assert_eq!(bytes.next().unwrap(), [0, 1, 0, 0, 1, 0, 1, 1]);
        // JZ
        assert_eq!(bytes.next().unwrap(), [1, 1, 0, 0, 1, 0, 1, 0]);
        // JZ address one
        assert_eq!(bytes.next().unwrap(), [0, 0, 0, 0, 0, 0, 0, 0]);
        // JZ address two
        assert_eq!(bytes.next().unwrap(), [0, 1, 0, 0, 1, 0, 1, 1]);
        // ADD
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 0, 0, 0, 0, 0]);
        // ADD
        assert_eq!(bytes.next().unwrap(), [1, 0, 0, 0, 0, 1, 1, 1]);
        // HLT
        assert_eq!(bytes.next().unwrap(), [0, 1, 1, 1, 0, 1, 1, 0]);
    }

    #[test]
    fn test_disassemble() {
        let assembler = Assembler::new("data/test/end_to_end.asm".to_owned(), "test_disassemble_binary".to_owned());
        assembler.assemble();

        let instructions = assembler.disassemble("test_disassemble_binary".to_owned());
        assert_eq!(instructions.len(), 50);

        let mut instruction = instructions.get(&0).unwrap();
        assert_eq!(
            *instruction,
            Instruction::IntermediateRegister(InstructionCommand::Mvi, 28, InstructionRegister::A)
        );

        instruction = instructions.get(&2).unwrap();
        assert_eq!(
            *instruction,
            Instruction::DoubleRegister(
                InstructionCommand::Mov,
                (InstructionRegister::A, InstructionRegister::B)
            )
        );

        instruction = instructions.get(&3).unwrap();
        assert_eq!(
            *instruction,
            Instruction::SingleRegister(InstructionCommand::Ana, InstructionRegister::B)
        );

        instruction = instructions.get(&4).unwrap();
        assert_eq!(
            *instruction,
            Instruction::SingleRegister(InstructionCommand::Add, InstructionRegister::A)
        );

        instruction = instructions.get(&5).unwrap();
        assert_eq!(
            *instruction,
            Instruction::SingleRegister(InstructionCommand::Sub, InstructionRegister::A)
        );

        instruction = instructions.get(&6).unwrap();
        assert_eq!(
            *instruction,
            Instruction::SingleRegister(InstructionCommand::Inr, InstructionRegister::A)
        );

        instruction = instructions.get(&7).unwrap();
        assert_eq!(
            *instruction,
            Instruction::SingleRegister(InstructionCommand::Dcr, InstructionRegister::A)
        );

        instruction = instructions.get(&8).unwrap();
        assert_eq!(
            *instruction,
            Instruction::Intermediate(InstructionCommand::Adi, -103)
        );

        instruction = instructions.get(&10).unwrap();
        assert_eq!(
            *instruction,
            Instruction::NoRegister(InstructionCommand::Stc)
        );

        instruction = instructions.get(&11).unwrap();
        assert_eq!(
            *instruction,
            Instruction::NoRegister(InstructionCommand::Cmc)
        );

        instruction = instructions.get(&12).unwrap();
        assert_eq!(
            *instruction,
            Instruction::NoRegister(InstructionCommand::Cma)
        );

        instruction = instructions.get(&13).unwrap();
        assert_eq!(
            *instruction,
            Instruction::SingleRegister(InstructionCommand::Adc, InstructionRegister::C)
        );

        instruction = instructions.get(&14).unwrap();
        assert_eq!(
            *instruction,
            Instruction::Intermediate(InstructionCommand::Aci, 12)
        );

        instruction = instructions.get(&16).unwrap();
        assert_eq!(
            *instruction,
            Instruction::Intermediate(InstructionCommand::Sui, 12)
        );

        instruction = instructions.get(&18).unwrap();
        assert_eq!(
            *instruction,
            Instruction::NoRegister(InstructionCommand::Rlc)
        );

        instruction = instructions.get(&19).unwrap();
        assert_eq!(
            *instruction,
            Instruction::NoRegister(InstructionCommand::Rrc)
        );

        instruction = instructions.get(&20).unwrap();
        assert_eq!(
            *instruction,
            Instruction::NoRegister(InstructionCommand::Ral)
        );

        instruction = instructions.get(&21).unwrap();
        assert_eq!(
            *instruction,
            Instruction::NoRegister(InstructionCommand::Rar)
        );

        instruction = instructions.get(&22).unwrap();
        assert_eq!(
            *instruction,
            Instruction::SingleRegister(InstructionCommand::Ora, InstructionRegister::B)
        );

        instruction = instructions.get(&23).unwrap();
        assert_eq!(
            *instruction,
            Instruction::NoRegister(InstructionCommand::Daa)
        );

        instruction = instructions.get(&24).unwrap();
        assert_eq!(
            *instruction,
            Instruction::PairRegister(InstructionCommand::Stax, InstructionRegisterPair::BC)
        );

        instruction = instructions.get(&25).unwrap();
        assert_eq!(
            *instruction,
            Instruction::PairRegister(InstructionCommand::Ldax, InstructionRegisterPair::DE)
        );

        instruction = instructions.get(&26).unwrap();
        assert_eq!(
            *instruction,
            Instruction::SingleRegister(InstructionCommand::Cmp, InstructionRegister::B)
        );

        instruction = instructions.get(&27).unwrap();
        assert_eq!(
            *instruction,
            Instruction::SingleRegister(InstructionCommand::Xra, InstructionRegister::B)
        );

        instruction = instructions.get(&28).unwrap();
        assert_eq!(
            *instruction,
            Instruction::SingleRegister(InstructionCommand::Sbb, InstructionRegister::B)
        );

        instruction = instructions.get(&29).unwrap();
        assert_eq!(
            *instruction,
            Instruction::NoRegister(InstructionCommand::Xchg)
        );

        instruction = instructions.get(&30).unwrap();
        assert_eq!(
            *instruction,
            Instruction::NoRegister(InstructionCommand::Sphl)
        );

        instruction = instructions.get(&31).unwrap();
        assert_eq!(
            *instruction,
            Instruction::NoRegister(InstructionCommand::Xthl)
        );

        instruction = instructions.get(&32).unwrap();
        assert_eq!(
            *instruction,
            Instruction::PairRegister(InstructionCommand::Dcx, InstructionRegisterPair::BC)
        );

        instruction = instructions.get(&33).unwrap();
        assert_eq!(
            *instruction,
            Instruction::PairRegister(InstructionCommand::Inx, InstructionRegisterPair::SP)
        );

        instruction = instructions.get(&34).unwrap();
        assert_eq!(
            *instruction,
            Instruction::PairRegister(InstructionCommand::Dad, InstructionRegisterPair::BC)
        );

        instruction = instructions.get(&35).unwrap();
        assert_eq!(
            *instruction,
            Instruction::PairRegister(InstructionCommand::Push, InstructionRegisterPair::FA)
        );

        instruction = instructions.get(&36).unwrap();
        assert_eq!(
            *instruction,
            Instruction::PairRegister(InstructionCommand::Pop, InstructionRegisterPair::FA)
        );

        instruction = instructions.get(&37).unwrap();
        assert_eq!(
            *instruction,
            Instruction::Intermediate(InstructionCommand::Ori, 15)
        );

        instruction = instructions.get(&39).unwrap();
        assert_eq!(
            *instruction,
            Instruction::Intermediate(InstructionCommand::Xri, 15)
        );

        instruction = instructions.get(&41).unwrap();
        assert_eq!(
            *instruction,
            Instruction::Intermediate(InstructionCommand::Ani, -128)
        );

        instruction = instructions.get(&43).unwrap();
        assert_eq!(
            *instruction,
            Instruction::Intermediate(InstructionCommand::Cpi, 15)
        );

        instruction = instructions.get(&45).unwrap();
        assert_eq!(
            *instruction,
            Instruction::Intermediate(InstructionCommand::Sbi, 0)
        );

        instruction = instructions.get(&47).unwrap();
        assert_eq!(
            *instruction,
            Instruction::Intermediate16Bit(
                InstructionCommand::Lxi,
                InstructionRegisterPair::SP,
                12345
            )
        );

        instruction = instructions.get(&50).unwrap();
        assert_eq!(
            *instruction,
            Instruction::Intermediate16BitNoReg(InstructionCommand::Sta, 42)
        );

        instruction = instructions.get(&53).unwrap();
        assert_eq!(
            *instruction,
            Instruction::Intermediate16BitNoReg(InstructionCommand::Lda, 0)
        );

        instruction = instructions.get(&56).unwrap();
        assert_eq!(
            *instruction,
            Instruction::Intermediate16BitNoReg(InstructionCommand::Shld, 12345)
        );

        instruction = instructions.get(&59).unwrap();
        assert_eq!(
            *instruction,
            Instruction::Intermediate16BitNoReg(InstructionCommand::Lhld, 4000)
        );

        instruction = instructions.get(&62).unwrap();
        assert_eq!(
            *instruction,
            Instruction::Label(InstructionCommand::Jmp, 75)
        );

        instruction = instructions.get(&65).unwrap();
        assert_eq!(*instruction, Instruction::Label(InstructionCommand::Jc, 75));

        instruction = instructions.get(&68).unwrap();
        assert_eq!(*instruction, Instruction::Label(InstructionCommand::Jnc, 75));

        instruction = instructions.get(&71).unwrap();
        assert_eq!(*instruction, Instruction::Label(InstructionCommand::Jz, 75));

        instruction = instructions.get(&74).unwrap();
        assert_eq!(
            *instruction,
            Instruction::SingleRegister(InstructionCommand::Add, InstructionRegister::B)
        );

        instruction = instructions.get(&75).unwrap();
        assert_eq!(
            *instruction,
            Instruction::SingleRegister(InstructionCommand::Add, InstructionRegister::A)
        );

        instruction = instructions.get(&76).unwrap();
        assert_eq!(
            *instruction,
            Instruction::NoRegister(InstructionCommand::Hlt)
        );
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

        let instructions = &assembler.parse_binary_instructions(&instruction);
        let instruction = instructions.get(&0).unwrap();

        assert_eq!(
            *instruction,
            Instruction::PairRegister(InstructionCommand::Stax, InstructionRegisterPair::BC)
        );

        let instruction = vec![vec![0, 0, 0, 1, 0, 0, 1, 0]];

        let instructions = &assembler.parse_binary_instructions(&instruction);
        let instruction = instructions.get(&0).unwrap();

        assert_eq!(
            *instruction,
            Instruction::PairRegister(InstructionCommand::Stax, InstructionRegisterPair::DE)
        );
    }

    #[test]
    fn test_ldax_parsing() {
        let assembler =
            Assembler::new("test.asm".to_owned(), "test_ldax_parsing_binary".to_owned());
        let instruction = vec![vec![0, 0, 0, 0, 1, 0, 1, 0]];

        let instructions = &assembler.parse_binary_instructions(&instruction);
        let instruction = instructions.get(&0).unwrap();

        assert_eq!(
            *instruction,
            Instruction::PairRegister(InstructionCommand::Ldax, InstructionRegisterPair::BC)
        );

        let instruction = vec![vec![0, 0, 0, 1, 1, 0, 1, 0]];

        let instructions = &assembler.parse_binary_instructions(&instruction);
        let instruction = instructions.get(&0).unwrap();

        assert_eq!(
            *instruction,
            Instruction::PairRegister(InstructionCommand::Ldax, InstructionRegisterPair::DE)
        );
    }
}
