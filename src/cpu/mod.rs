use crate::assembler::{
    Instruction, InstructionCommand, InstructionRegister, InstructionRegisterPair,
};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub fn initialize_cpu() -> Cpu {
    Cpu {
        registers: vec![0; 8],
        memory: vec![0; 65536],
        stack_pointer: 0,
        flags: vec![false; 8],
        program_counter: 0,
    }
}

#[derive(Debug)]
pub struct Cpu {
    registers: Vec<i8>,
    memory: Vec<i8>,
    stack_pointer: u16,

    // S Z x A x P x C
    flags: Vec<bool>,
    program_counter: u16,
}
#[derive(Debug, EnumIter, Clone)]
enum Flag {
    S,
    Z,
    A,
    P,
    C,
}

impl Flag {
    pub fn get_index(&self) -> usize {
        match self {
            Flag::S => 0,
            Flag::Z => 1,
            Flag::A => 3,
            Flag::P => 5,
            Flag::C => 7,
        }
    }
}

impl Cpu {
    fn get_register(&self, register: InstructionRegister) -> i8 {
        self.registers[register.to_index() as usize]
    }

    fn change_register(&mut self, register: InstructionRegister, value: i8) {
        self.registers[register.to_index() as usize] = value;
    }

    fn set_memory(&mut self, address: u16, value: i8) {
        self.memory[address as usize] = value;
    }

    fn get_memory(&self, address: u16) -> i8 {
        self.memory[address as usize]
    }

    fn set_stack_pointer(&mut self, value: u16) {
        self.stack_pointer = value;
    }

    fn get_stack_pointer(&self) -> u16 {
        self.stack_pointer
    }

    fn get_program_counter(&self) -> u16 {
        self.program_counter
    }

    fn set_program_counter(&mut self, value: u16) {
        self.program_counter = value;
    }

    fn incr_program_counter(&mut self, instruction: &Instruction) {
        self.set_program_counter(self.get_program_counter() + instruction.get_size() as u16);
    }

    pub fn run(&mut self, instructions: HashMap<u16, Instruction>, printing: bool) {
        if printing {
            self.print_run(instructions);
            return;
        }

        let mut instruction: &Instruction;

        loop {
            instruction = instructions.get(&self.get_program_counter()).unwrap();

            if let Instruction::NoRegister(command) = instruction {
                if matches!(command, InstructionCommand::Hlt) {
                    self.incr_program_counter(instruction);
                    println!("Execution finished");

                    println!("Final status: ");
                    self.print_status();
                    return;
                }
            }

            self.execute(instruction);

            if !matches!(instruction, Instruction::Label(_, _)) {
                self.incr_program_counter(instruction);
            }
        }
    }

    pub fn print_run(&mut self, instructions: HashMap<u16, Instruction>) {
        println!("Initial status:");
        self.print_status();

        let mut instruction: &Instruction;
        loop {
            instruction = instructions.get(&self.get_program_counter()).unwrap();

            println!("-------------");
            println!("{:?}", instruction);

            self.execute(instruction);
            self.incr_program_counter(instruction);

            if let Instruction::NoRegister(command) = instruction {
                if matches!(command, InstructionCommand::Hlt) {
                    println!("Execution finished");
                    println!("Final status: ");
                    self.print_status();
                    return;
                }
            }

            self.print_status();
        }
    }

    fn execute(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::NoRegister(command) => self.execute_no_reg_instruction(command),
            Instruction::SingleRegister(command, register) => {
                self.execute_single_reg_instruction(command, register)
            }
            Instruction::DoubleRegister(command, registers) => {
                self.execute_double_reg_instruction(command, registers)
            }
            Instruction::Intermediate(command, intermediate) => {
                self.execute_intermediate_instruction(command, *intermediate)
            }
            Instruction::Intermediate16Bit(command, register_pair, intermediate) => {
                self.execute_intermediate_16_bit_instruction(command, register_pair, *intermediate)
            }
            Instruction::Intermediate16BitNoReg(command, intermediate) => {
                self.execute_intermediate_16_bit_instruction_no_reg(command, *intermediate)
            }
            Instruction::IntermediateRegister(command, intermediate, register) => {
                self.execute_intermediate_reg_instruction(command, register, *intermediate)
            }
            Instruction::PairRegister(command, register_pair) => {
                self.execute_pair_reg_instruction(command, register_pair)
            }
            Instruction::Label(command, address) => {
                self.execute_label_instruction(command, *address)
            }
        }
    }

    fn execute_no_reg_instruction(&mut self, command: &InstructionCommand) {
        match command {
            InstructionCommand::Stc => self.execute_stc(),
            InstructionCommand::Cmc => self.execute_cmc(),
            InstructionCommand::Cma => self.execute_cma(),
            InstructionCommand::Rlc => self.execute_rlc(),
            InstructionCommand::Rrc => self.execute_rrc(),
            InstructionCommand::Ral => self.execute_ral(),
            InstructionCommand::Rar => self.execute_rar(),
            InstructionCommand::Daa => self.execute_daa(),
            InstructionCommand::Xchg => self.execute_xchg(),
            InstructionCommand::Sphl => self.execute_sphl(),
            InstructionCommand::Xthl => self.execute_xthl(),
            InstructionCommand::Pchl => self.execute_pchl(),
            _ => panic!("invalid instruction"),
        }
    }

    fn execute_single_reg_instruction(
        &mut self,
        command: &InstructionCommand,
        register: &InstructionRegister,
    ) {
        match command {
            InstructionCommand::Add => self.execute_add(register),
            InstructionCommand::Adc => self.execute_adc(register),
            InstructionCommand::Sub => self.execute_sub(register),
            InstructionCommand::Inr => self.execute_inr(register),
            InstructionCommand::Dcr => self.execute_dcr(register),
            InstructionCommand::Ana => self.execute_ana(register),
            InstructionCommand::Ora => self.execute_ora(register),
            InstructionCommand::Cmp => self.execute_cmp(register),
            InstructionCommand::Xra => self.execute_xra(register),
            InstructionCommand::Sbb => self.execute_sbb(register),
            _ => panic!("invalid instruction"),
        }
    }

    fn execute_double_reg_instruction(
        &mut self,
        command: &InstructionCommand,
        registers: &(InstructionRegister, InstructionRegister),
    ) {
        match command {
            InstructionCommand::Mov => self.execute_mov(registers),
            _ => panic!("invalid instruction"),
        }
    }

    fn execute_intermediate_instruction(&mut self, command: &InstructionCommand, intermediate: i8) {
        match command {
            InstructionCommand::Adi => self.execute_adi(intermediate),
            InstructionCommand::Aci => self.execute_aci(intermediate),
            InstructionCommand::Sui => self.execute_sui(intermediate),
            InstructionCommand::Ori => self.execute_ori(intermediate),
            InstructionCommand::Xri => self.execute_xri(intermediate),
            InstructionCommand::Ani => self.execute_ani(intermediate),
            InstructionCommand::Cpi => self.execute_cpi(intermediate),
            InstructionCommand::Sbi => self.execute_sbi(intermediate),
            _ => panic!("invalid instruction"),
        }
    }

    fn execute_intermediate_16_bit_instruction(
        &mut self,
        command: &InstructionCommand,
        register_pair: &InstructionRegisterPair,
        intermediate: i16,
    ) {
        match command {
            InstructionCommand::Lxi => self.execute_lxi(register_pair, intermediate),
            _ => panic!("invalid instruction"),
        }
    }

    fn execute_intermediate_16_bit_instruction_no_reg(
        &mut self,
        command: &InstructionCommand,
        intermediate: i16,
    ) {
        match command {
            InstructionCommand::Sta => self.execute_sta(intermediate),
            InstructionCommand::Lda => self.execute_lda(intermediate),
            InstructionCommand::Shld => self.execute_shld(intermediate),
            InstructionCommand::Lhld => self.execute_lhld(intermediate),
            _ => panic!("invalid instruction"),
        }
    }

    fn execute_intermediate_reg_instruction(
        &mut self,
        command: &InstructionCommand,
        register: &InstructionRegister,
        intermediate: i8,
    ) {
        match command {
            InstructionCommand::Mvi => self.execute_mvi(register, intermediate),
            _ => panic!("invalid instruction"),
        }
    }

    fn execute_pair_reg_instruction(
        &mut self,
        command: &InstructionCommand,
        register_pair: &InstructionRegisterPair,
    ) {
        match command {
            InstructionCommand::Stax => self.execute_stax(register_pair),
            InstructionCommand::Ldax => self.execute_ldax(register_pair),
            InstructionCommand::Dcx => self.execute_dcx(register_pair),
            InstructionCommand::Inx => self.execute_inx(register_pair),
            InstructionCommand::Dad => self.execute_dad(register_pair),
            InstructionCommand::Push => self.execute_push(register_pair),
            InstructionCommand::Pop => self.execute_pop(register_pair),
            _ => panic!("invalid instruction"),
        }
    }

    fn execute_label_instruction(&mut self, command: &InstructionCommand, address: u16) {
        match command {
            InstructionCommand::Jmp => self.execute_jmp(address),
            _ => panic!("invalid instruction"),
        }
    }

    fn execute_mvi(&mut self, arg: &InstructionRegister, intermediate: i8) {
        self.change_register(*arg, intermediate);
    }

    fn execute_adi(&mut self, intermediate: i8) {
        let current_a = self.get_register(InstructionRegister::A);
        let new_a = current_a.wrapping_add(intermediate);

        if self.get_register(InstructionRegister::A) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(InstructionRegister::A) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }

        // if onecomplement representation added > 255 -> carry exists
        // example: 127 + 127
        // "x as u8 as u16" converts to onecomplement representation
        if (intermediate as u8 as u16) + (current_a as u8 as u16) > 255 {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        self.change_register(InstructionRegister::A, new_a);
    }

    fn execute_aci(&mut self, intermediate: i8) {
        let current_a = self.get_register(InstructionRegister::A);
        let new_a = current_a
            .wrapping_add(intermediate)
            .wrapping_add(self.get_flag(Flag::C) as i8);

        if self.get_register(InstructionRegister::A) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(InstructionRegister::A) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }

        // if onecomplement representation added > 255 -> carry exists
        // example: 127 + 127
        // "x as u8 as u16" converts to onecomplement representation
        if (intermediate as u8 as u16) + (current_a as u8 as u16) + (self.get_flag(Flag::C) as u16)
            > 255
        {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        self.change_register(InstructionRegister::A, new_a);
    }

    fn execute_sui(&mut self, intermediate: i8) {
        let current_a = self.get_register(InstructionRegister::A);
        let new_a = current_a.wrapping_sub(intermediate);

        self.change_register(InstructionRegister::A, new_a);

        if self.get_register(InstructionRegister::A) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(InstructionRegister::A) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }

        // if onecomplement representation subtraction < 0 -> set carry
        // "x as u8 as u16" converts to onecomplement representation
        if (current_a as u8 as u16).checked_sub(intermediate as u8 as u16) != None {
            self.set_flag(Flag::C, false);
        } else {
            self.set_flag(Flag::C, true);
        }
    }

    fn execute_mov(&mut self, args: &(InstructionRegister, InstructionRegister)) {
        let source_value = self.get_register(args.0);

        self.change_register(args.1, source_value);
    }

    fn execute_add(&mut self, arg: &InstructionRegister) {
        let source_value = self.get_register(*arg);
        let current_a = self.get_register(InstructionRegister::A);
        let new_a = current_a.wrapping_add(source_value);

        self.change_register(InstructionRegister::A, new_a);

        if self.get_register(InstructionRegister::A) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(InstructionRegister::A) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }

        // if onecomplement representation added > 255 -> carry exists
        // example: 127 + 127
        // "x as u8 as u16" converts to onecomplement representation
        if (source_value as u8 as u16) + (current_a as u8 as u16) > 255 {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        self.change_register(InstructionRegister::A, new_a);
    }

    fn execute_adc(&mut self, arg: &InstructionRegister) {
        let source_value = self.get_register(*arg);
        let current_a = self.get_register(InstructionRegister::A);

        let new_a = current_a + source_value + self.get_flag(Flag::C) as i8;

        if self.get_register(InstructionRegister::A) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(InstructionRegister::A) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }

        // if onecomplement representation added > 255 -> carry exists
        // example: 127 + 127
        // "x as u8 as u16" converts to onecomplement representation
        if (source_value as u8 as u16) + (current_a as u8 as u16) + self.get_flag(Flag::C) as u16
            > 255
        {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        self.change_register(InstructionRegister::A, new_a);
    }

    fn execute_sub(&mut self, args: &InstructionRegister) {
        let source_value = self.get_register(*args);
        let current_a = self.get_register(InstructionRegister::A);
        let new_a = current_a.wrapping_sub(source_value);

        self.change_register(InstructionRegister::A, new_a);

        if self.get_register(InstructionRegister::A) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(InstructionRegister::A) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }

        // if onecomplement representation subtraction < 0 -> set carry
        // "x as u8 as u16" converts to onecomplement representation
        if (current_a as u8 as u16).checked_sub(source_value as u8 as u16) != None {
            self.set_flag(Flag::C, false);
        } else {
            self.set_flag(Flag::C, true);
        }
    }

    fn execute_inr(&mut self, arg: &InstructionRegister) {
        let new_value = self.get_register(*arg) + 1;

        self.change_register(*arg, new_value);

        if self.get_register(InstructionRegister::A) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(InstructionRegister::A) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }
    }

    fn execute_dcr(&mut self, arg: &InstructionRegister) {
        let new_value = self.get_register(*arg) - 1;

        self.change_register(*arg, new_value);

        if self.get_register(InstructionRegister::A) == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if self.get_register(InstructionRegister::A) < 0 {
            self.set_flag(Flag::S, true);
        } else {
            self.set_flag(Flag::S, false);
        }
    }

    fn execute_ana(&mut self, arg: &InstructionRegister) {
        let acc = self.get_register(InstructionRegister::A);
        let reg = self.get_register(*arg);

        self.change_register(InstructionRegister::A, acc & reg);
    }

    fn set_flag(&mut self, flag: Flag, value: bool) {
        self.flags[flag.get_index()] = value;
    }

    fn get_flag(&self, flag: Flag) -> bool {
        self.flags[flag.get_index()]
    }

    fn execute_stc(&mut self) {
        self.set_flag(Flag::C, true);
    }

    fn execute_cmc(&mut self) {
        self.set_flag(Flag::C, !self.get_flag(Flag::C));
    }

    fn execute_cma(&mut self) {
        // complement of twos-complement is always
        // -(num+1)

        self.change_register(
            InstructionRegister::A,
            !self.get_register(InstructionRegister::A),
        );
    }

    // last bit can never be 1 after shift
    // that's why the case of carry=0 and last bit 1 can be ignored
    fn execute_rlc(&mut self) {
        let mut acc = self.get_register(InstructionRegister::A);
        if (acc >> 7) & 1 == 1 {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        acc <<= 1;

        if self.get_flag(Flag::C) {
            acc |= 1;
        }

        self.change_register(InstructionRegister::A, acc);
    }

    fn execute_rrc(&mut self) {
        let mut acc = self.get_register(InstructionRegister::A);
        if (acc << 7) & -128 == -128 {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        // convert to u8 to make sure LSR is used
        // otherwise most significant bit is 1 after shift
        acc = ((acc as u8) >> 1) as i8;

        if self.get_flag(Flag::C) {
            acc |= -128;
        }

        self.change_register(InstructionRegister::A, acc);
    }

    fn execute_ral(&mut self) {
        let mut acc = self.get_register(InstructionRegister::A);
        let flag = self.get_flag(Flag::C);
        if (acc >> 7) & 1 == 1 {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        acc <<= 1;

        if flag {
            acc |= 1;
        }

        self.change_register(InstructionRegister::A, acc);
    }

    fn execute_rar(&mut self) {
        let mut acc = self.get_register(InstructionRegister::A);
        let flag = self.get_flag(Flag::C);
        if (acc << 7) & -128 == -128 {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }

        // convert to u8 to make sure LSR is used
        // otherwise most significant bit is 1 after shift
        acc = ((acc as u8) >> 1) as i8;

        if flag {
            acc |= -128;
        }

        self.change_register(InstructionRegister::A, acc);
    }

    fn execute_ora(&mut self, arg: &InstructionRegister) {
        let mut acc = self.get_register(InstructionRegister::A);
        acc |= self.get_register(*arg);

        self.change_register(InstructionRegister::A, acc);
        self.set_flag(Flag::C, false);
    }

    fn execute_daa(&mut self) {
        let mut acc = self.get_register(InstructionRegister::A);

        // check if 4 least significant bits are > 9
        if (acc & 15) > 9 || self.get_flag(Flag::A) {
            acc = acc.wrapping_add(6);

            // check if carry out happens
            if (self.get_register(InstructionRegister::A) & -16) != (acc & -16) {
                self.set_flag(Flag::A, true);
            } else {
                self.set_flag(Flag::A, false);
            }
        }

        // check if 4 most significant bits are > 9
        // increment 4 most significant bits by 6
        // since its 4 most significant, +6 = +96
        let most_significant_bits = (((acc & -16) as u8) >> 4) as i8;
        if most_significant_bits > 9 || self.get_flag(Flag::C) {
            // if onecomplement representation added > 255 -> carry exists
            // example: 127 + 127
            // "x as u8 as u16" converts to onecomplement representation
            if (acc as u8 as u16) + (96u16) > 255 {
                self.set_flag(Flag::C, true);
            } else {
                self.set_flag(Flag::C, false);
            }
            acc = acc.wrapping_add(96);
        }

        self.change_register(InstructionRegister::A, acc);
    }

    fn execute_stax(&mut self, register_pair: &InstructionRegisterPair) {
        let registers = register_pair.get_registers();
        let mut first_register = self.get_register(registers.0) as u16;
        let mut second_register = self.get_register(registers.1) as u16;
        let acc = self.get_register(InstructionRegister::A);

        // make sure first 8 bits are 0 because of negative numbers
        second_register &= 255;

        first_register <<= 8;

        let address = first_register | second_register;
        self.set_memory(address, acc);
    }

    fn execute_ldax(&mut self, register_pair: &InstructionRegisterPair) {
        let registers = register_pair.get_registers();
        let mut first_register = self.get_register(registers.0) as u16;
        let mut second_register = self.get_register(registers.1) as u16;

        // make sure first 8 bits are 0 because of negative numbers
        second_register &= 255;

        first_register <<= 8;

        let address = first_register | second_register;
        self.change_register(InstructionRegister::A, self.get_memory(address));
    }

    fn execute_cmp(&mut self, register: &InstructionRegister) {
        let acc = self.get_register(InstructionRegister::A);
        let reg = self.get_register(*register);

        let result = acc.wrapping_sub(reg);

        if result == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        // "x as u8 as u16" converts to onecomplement representation
        // if onecomplement representation subtraction < 0 -> carry happens
        // only works if subtraction is happening, if reg is negative,
        // comparision with 255 has to be done
        if reg < 0 {
            if ((acc as u8 as u16) + (reg as u8 as u16)) > 255 {
                self.set_flag(Flag::C, false);
            } else {
                self.set_flag(Flag::C, true);
            }
        } else if (acc as u8 as u16).checked_sub(reg as u8 as u16) == None {
            self.set_flag(Flag::C, false);
        } else {
            self.set_flag(Flag::C, true);
        }
    }

    fn execute_xra(&mut self, register: &InstructionRegister) {
        let acc = self.get_register(InstructionRegister::A);
        let reg = self.get_register(*register);

        let result = acc ^ reg;

        if result == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        self.change_register(*register, result);
    }

    fn execute_sbb(&mut self, register: &InstructionRegister) {
        let acc = self.get_register(InstructionRegister::A);
        let mut reg = self.get_register(*register);

        reg = reg.wrapping_add(self.get_flag(Flag::C) as i8);

        let result = acc.wrapping_sub(reg);

        if result == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if (acc as u8).checked_add(-reg as u8) == None {
            self.set_flag(Flag::C, false);
        } else {
            self.set_flag(Flag::C, true);
        }

        self.change_register(InstructionRegister::A, result);
    }

    fn execute_xchg(&mut self) {
        let reg_d = self.get_register(InstructionRegister::D);
        let reg_e = self.get_register(InstructionRegister::E);
        let reg_h = self.get_register(InstructionRegister::H);
        let reg_l = self.get_register(InstructionRegister::L);

        self.change_register(InstructionRegister::D, reg_h);
        self.change_register(InstructionRegister::E, reg_l);
        self.change_register(InstructionRegister::H, reg_d);
        self.change_register(InstructionRegister::L, reg_e);
    }

    fn execute_sphl(&mut self) {
        let mut reg_h = self.get_register(InstructionRegister::H) as u16;
        let mut reg_l = self.get_register(InstructionRegister::L) as u16;

        // make sure first 8 bits are 0 because of negative numbers
        reg_l &= 255;

        reg_h <<= 8;

        let stack_pointer = reg_l | reg_h;
        self.set_stack_pointer(stack_pointer);
    }

    fn execute_xthl(&mut self) {
        let reg_l = self.get_register(InstructionRegister::L);
        let reg_h = self.get_register(InstructionRegister::H);

        let memory = self.get_memory(self.get_stack_pointer());
        let memory_incr = self.get_memory(self.get_stack_pointer() + 1);

        self.change_register(InstructionRegister::L, memory);
        self.change_register(InstructionRegister::H, memory_incr);
        self.set_memory(self.get_stack_pointer(), reg_l);
        self.set_memory(self.get_stack_pointer() + 1, reg_h);
    }

    fn execute_dcx(&mut self, register_pair: &InstructionRegisterPair) {
        if matches!(register_pair, InstructionRegisterPair::SP) {
            self.set_stack_pointer(self.get_stack_pointer().wrapping_sub(1));
            return;
        }

        let registers = register_pair.get_registers();

        let mut first_register = self.get_register(registers.0) as u16;
        let mut second_register = self.get_register(registers.1) as u16;

        // make sure first 8 bits are 0 because of negative numbers
        second_register &= 255;

        first_register <<= 8;

        let mut value = first_register | second_register;
        value = value.wrapping_sub(1);

        self.change_register(registers.0, (value >> 8) as i8);
        self.change_register(registers.1, (value & 255) as i8);
    }

    fn execute_inx(&mut self, register_pair: &InstructionRegisterPair) {
        if matches!(register_pair, InstructionRegisterPair::SP) {
            self.set_stack_pointer(self.get_stack_pointer().wrapping_add(1));
            return;
        }

        let registers = register_pair.get_registers();

        let mut first_register = self.get_register(registers.0) as u16;
        let mut second_register = self.get_register(registers.1) as u16;

        // make sure first 8 bits are 0 because of negative numbers
        second_register &= 255;

        first_register <<= 8;

        let mut value = first_register | second_register;
        value = value.wrapping_add(1);

        self.change_register(registers.0, (value >> 8) as i8);
        self.change_register(registers.1, (value & 255) as i8);
    }

    fn execute_dad(&mut self, register_pair: &InstructionRegisterPair) {
        let value: u16;
        if matches!(register_pair, InstructionRegisterPair::SP) {
            value = self.get_stack_pointer();
        } else {
            let registers = register_pair.get_registers();

            let mut first_register = self.get_register(registers.0) as u16;
            let mut second_register = self.get_register(registers.1) as u16;

            // make sure first 8 bits are 0 because of negative numbers
            second_register &= 255;

            first_register <<= 8;

            value = first_register | second_register;
        }

        let mut h_register = self.get_register(InstructionRegister::H) as u16;
        let mut l_register = self.get_register(InstructionRegister::L) as u16;

        // make sure first 8 bits are 0 because of negative numbers
        l_register &= 255;

        h_register <<= 8;

        let hl_value = h_register | l_register;
        let result = value.wrapping_add(hl_value);

        self.change_register(InstructionRegister::H, (result >> 8) as i8);
        self.change_register(InstructionRegister::L, (result & 255) as i8);

        if ((value as u32) + (hl_value as u32)) > 65535 {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }
    }

    fn execute_push(&mut self, register_pair: &InstructionRegisterPair) {
        let first_register: i8;
        let mut second_register: i8;

        if matches!(register_pair, InstructionRegisterPair::FA) {
            first_register = self.get_register(InstructionRegister::A);

            second_register = 2;
            second_register |= (self.get_flag(Flag::S) as i8) << 7;
            second_register |= (self.get_flag(Flag::Z) as i8) << 6;
            second_register |= (self.get_flag(Flag::A) as i8) << 4;
            second_register |= (self.get_flag(Flag::P) as i8) << 2;
            second_register |= self.get_flag(Flag::C) as i8;
        } else {
            let registers = register_pair.get_registers();
            first_register = self.get_register(registers.0);
            second_register = self.get_register(registers.1);
        }

        let stack_pointer = self.get_stack_pointer();

        self.set_memory(stack_pointer.wrapping_sub(1), first_register);
        self.set_memory(stack_pointer.wrapping_sub(2), second_register);
        self.set_stack_pointer(stack_pointer.wrapping_sub(2));
    }

    fn execute_pop(&mut self, register_pair: &InstructionRegisterPair) {
        if !matches!(register_pair, &InstructionRegisterPair::FA) {
            let registers = register_pair.get_registers();

            let stack_pointer = self.get_stack_pointer();
            self.change_register(registers.1, self.get_memory(self.get_stack_pointer()));
            self.change_register(
                registers.0,
                self.get_memory(self.get_stack_pointer().wrapping_add(1)),
            );
            self.set_stack_pointer(stack_pointer.wrapping_add(2));
            return;
        }

        let stack_pointer = self.get_stack_pointer();
        self.change_register(
            InstructionRegister::A,
            self.get_memory(stack_pointer.wrapping_add(1)),
        );

        let flags = self.get_memory(stack_pointer);
        self.set_flag(Flag::S, (flags >> 7) != 0);
        self.set_flag(Flag::Z, ((flags >> 6) & 1) != 0);
        self.set_flag(Flag::A, ((flags >> 4) & 1) != 0);
        self.set_flag(Flag::P, ((flags >> 2) & 1) != 0);
        self.set_flag(Flag::C, (flags & 1) != 0);
        self.set_stack_pointer(stack_pointer.wrapping_add(2));
    }

    fn execute_ori(&mut self, intermediate: i8) {
        let mut acc = self.get_register(InstructionRegister::A);
        acc |= intermediate;

        self.change_register(InstructionRegister::A, acc);
        self.set_flag(Flag::C, false);

        if acc == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }
    }

    fn execute_xri(&mut self, intermediate: i8) {
        let mut acc = self.get_register(InstructionRegister::A);
        acc ^= intermediate;

        self.change_register(InstructionRegister::A, acc);
        self.set_flag(Flag::C, false);

        if acc == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }
    }

    fn execute_ani(&mut self, intermediate: i8) {
        let acc = self.get_register(InstructionRegister::A);
        let result = acc & intermediate;

        self.change_register(InstructionRegister::A, result);
        self.set_flag(Flag::C, false);

        if result == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }
    }

    fn execute_cpi(&mut self, intermediate: i8) {
        let acc = self.get_register(InstructionRegister::A);

        let result = acc.wrapping_sub(intermediate);

        if result == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if (intermediate < 0 && acc >= 0) || (intermediate >= 0 && acc < 0) {
            if self.get_flag(Flag::C) {
                self.set_flag(Flag::C, false);
            } else {
                self.set_flag(Flag::C, true);
            }

            return;
        }

        if intermediate > acc {
            self.set_flag(Flag::C, true);
        } else {
            self.set_flag(Flag::C, false);
        }
    }

    fn execute_sbi(&mut self, mut intermediate: i8) {
        let acc = self.get_register(InstructionRegister::A);

        intermediate = intermediate.wrapping_add(self.get_flag(Flag::C) as i8);

        let result = acc.wrapping_sub(intermediate);

        if result == 0 {
            self.set_flag(Flag::Z, true);
        } else {
            self.set_flag(Flag::Z, false);
        }

        if (acc as u8).checked_add(-intermediate as u8) == None {
            self.set_flag(Flag::C, false);
        } else {
            self.set_flag(Flag::C, true);
        }

        self.change_register(InstructionRegister::A, result);
    }

    fn execute_lxi(&mut self, register_pair: &InstructionRegisterPair, intermediate: i16) {
        if matches!(register_pair, &InstructionRegisterPair::SP) {
            self.set_stack_pointer(intermediate as u16);
            return;
        }
        let registers = register_pair.get_registers();

        self.change_register(registers.0, (intermediate >> 8) as i8);
        self.change_register(registers.1, (intermediate & 255) as i8);
    }

    fn execute_sta(&mut self, intermediate: i16) {
        self.set_memory(
            intermediate as u16,
            self.get_register(InstructionRegister::A),
        );
    }

    fn execute_lda(&mut self, intermediate: i16) {
        self.change_register(InstructionRegister::A, self.get_memory(intermediate as u16))
    }

    fn execute_shld(&mut self, intermediate: i16) {
        self.set_memory(
            intermediate as u16,
            self.get_register(InstructionRegister::L),
        );
        self.set_memory(
            intermediate as u16 + 1,
            self.get_register(InstructionRegister::H),
        );
    }

    fn execute_lhld(&mut self, intermediate: i16) {
        self.change_register(InstructionRegister::L, self.get_memory(intermediate as u16));
        self.change_register(
            InstructionRegister::H,
            self.get_memory(intermediate as u16 + 1),
        );
    }

    fn execute_pchl(&mut self) {
        let reg_h = self.get_register(InstructionRegister::H) as i16;
        let reg_l = self.get_register(InstructionRegister::L) as i16;
        let counter = (reg_h << 8) + (reg_l & 255);

        self.set_program_counter(counter as u16);
    }

    fn execute_jmp(&mut self, address: u16) {
        self.set_program_counter(address);
    }

    fn print_status(&self) {
        for i in 0..7 {
            println!(
                "{}: {:#010b} ({})",
                i,
                self.get_register(InstructionRegister::from_index(i)),
                self.get_register(InstructionRegister::from_index(i))
            );
        }
        self.print_flags();
        self.print_stack_pointer();
        self.print_program_counter();
        self.print_memory();
    }

    fn print_flags(&self) {
        println!("Flags:");
        for flag in Flag::iter() {
            println!("{:?}: {}", flag.clone(), self.get_flag(flag));
        }
    }

    fn print_memory(&self) {
        println!("Memory:");
        for (address, value) in self.memory.iter().enumerate() {
            if *value != 0 {
                println!("{}: {}", address, value);
            }
        }
    }

    fn print_stack_pointer(&self) {
        println!("Stack Pointer: {}", self.get_stack_pointer());
    }

    fn print_program_counter(&self) {
        println!("Program counter: {}", self.get_program_counter());
    }
}

#[cfg(test)]
mod tests {
    use super::initialize_cpu;
    use crate::assembler;
    use crate::cpu::{Flag, InstructionRegister, InstructionRegisterPair};

    #[test]
    fn test_execute_end_to_end() {
        let mut cpu = initialize_cpu();

        let assembler = assembler::Assembler::new("test.asm".to_owned(), "output".to_owned());

        assembler.assemble();
        let instructions = assembler.disassemble("output".to_owned());

        cpu.run(instructions, false);

        assert_eq!(cpu.get_register(InstructionRegister::A), -56);
        assert_eq!(cpu.get_register(InstructionRegister::B), 27);
        assert_eq!(cpu.get_register(InstructionRegister::C), -1);
        assert_eq!(cpu.get_register(InstructionRegister::D), 0);
        assert_eq!(cpu.get_register(InstructionRegister::E), 0);
        assert_eq!(cpu.get_register(InstructionRegister::H), 0);
        assert_eq!(cpu.get_register(InstructionRegister::L), 0);

        assert_eq!(cpu.get_flag(Flag::S), true);
        assert_eq!(cpu.get_flag(Flag::Z), false);
        assert_eq!(cpu.get_flag(Flag::A), true);
        assert_eq!(cpu.get_flag(Flag::P), false);
        assert_eq!(cpu.get_flag(Flag::C), true);

        assert_eq!(cpu.get_stack_pointer(), 12345);
        assert_eq!(cpu.get_memory(7168), -124);
        assert_eq!(cpu.get_memory(0), -28);
        assert_eq!(cpu.get_memory(65535), 18);
        assert_eq!(cpu.get_memory(42), 127);
        assert_eq!(cpu.get_memory(12345), -1);
        assert_eq!(cpu.get_memory(12346), 27);
        assert_eq!(cpu.get_program_counter(), 68);
    }

    #[test]
    fn test_execute_mvi() {
        let mut cpu = initialize_cpu();

        cpu.execute_mvi(&InstructionRegister::A, 14);
        assert_eq!(cpu.get_register(InstructionRegister::A), 14);
    }

    #[test]
    fn test_execute_mov() {
        let mut cpu = initialize_cpu();
        cpu.change_register(InstructionRegister::A, 10);

        cpu.execute_mov(&(InstructionRegister::A, InstructionRegister::B));
        assert_eq!(cpu.get_register(InstructionRegister::B), 10);
    }

    #[test]
    fn test_execute_add() {
        let mut cpu = initialize_cpu();
        cpu.change_register(InstructionRegister::A, 5);
        cpu.set_flag(Flag::Z, true);

        cpu.execute_add(&InstructionRegister::A);
        assert_eq!(cpu.get_register(InstructionRegister::A), 10);
        assert_eq!(cpu.get_flag(Flag::Z), false);

        cpu.change_register(InstructionRegister::A, -5);
        cpu.execute_add(&InstructionRegister::A);
        assert_eq!(cpu.get_register(InstructionRegister::A), -10);
        assert_eq!(cpu.get_flag(Flag::S), true);

        cpu.change_register(InstructionRegister::A, 127);
        cpu.change_register(InstructionRegister::B, 127);
        cpu.set_flag(Flag::C, true);
        cpu.execute_add(&InstructionRegister::B);
        assert_eq!(cpu.get_register(InstructionRegister::A), -2);
        assert_eq!(cpu.get_flag(Flag::C), false);

        cpu.change_register(InstructionRegister::A, -64);
        cpu.change_register(InstructionRegister::B, 64);
        cpu.execute_add(&InstructionRegister::B);
        assert_eq!(cpu.get_register(InstructionRegister::A), 0);
        assert_eq!(cpu.get_flag(Flag::C), true);
    }

    #[test]
    fn test_execute_adc() {
        let mut cpu = initialize_cpu();
        cpu.change_register(InstructionRegister::A, 5);
        cpu.set_flag(Flag::Z, true);

        cpu.change_register(InstructionRegister::A, 10);
        cpu.set_flag(Flag::C, false);
        cpu.execute_adc(&InstructionRegister::A);
        assert_eq!(cpu.get_register(InstructionRegister::A), 20);

        cpu.change_register(InstructionRegister::A, 10);
        cpu.set_flag(Flag::C, true);
        cpu.execute_adc(&InstructionRegister::A);
        assert_eq!(cpu.get_register(InstructionRegister::A), 21);

        cpu.change_register(InstructionRegister::A, -64);
        cpu.change_register(InstructionRegister::B, 63);
        cpu.set_flag(Flag::C, true);
        cpu.execute_adc(&InstructionRegister::B);
        assert_eq!(cpu.get_register(InstructionRegister::A), 0);
        assert_eq!(cpu.get_flag(Flag::C), true);

        cpu.change_register(InstructionRegister::A, 15);
        cpu.change_register(InstructionRegister::B, 63);
        cpu.set_flag(Flag::C, true);
        cpu.execute_adc(&InstructionRegister::B);
        assert_eq!(cpu.get_register(InstructionRegister::A), 79);
        assert_eq!(cpu.get_flag(Flag::C), false);
    }

    #[test]
    fn test_execute_adi() {
        let mut cpu = initialize_cpu();
        cpu.change_register(InstructionRegister::A, 5);
        cpu.set_flag(Flag::Z, true);

        cpu.execute_adi(5);
        assert_eq!(cpu.get_register(InstructionRegister::A), 10);
        assert_eq!(cpu.get_flag(Flag::Z), false);

        cpu.change_register(InstructionRegister::A, -5);
        cpu.execute_adi(-5);
        assert_eq!(cpu.get_register(InstructionRegister::A), -10);
        assert_eq!(cpu.get_flag(Flag::S), true);

        cpu.change_register(InstructionRegister::A, -64);
        cpu.set_flag(Flag::C, true);
        cpu.execute_adi(64);
        assert_eq!(cpu.get_register(InstructionRegister::A), 0);
        assert_eq!(cpu.get_flag(Flag::C), true);

        cpu.change_register(InstructionRegister::A, 127);
        cpu.execute_adi(127);
        assert_eq!(cpu.get_register(InstructionRegister::A), -2);
        assert_eq!(cpu.get_flag(Flag::C), false);
    }

    #[test]
    fn test_execute_aci() {
        let mut cpu = initialize_cpu();
        cpu.change_register(InstructionRegister::A, 5);
        cpu.set_flag(Flag::Z, true);

        cpu.execute_aci(5);
        assert_eq!(cpu.get_register(InstructionRegister::A), 10);
        assert_eq!(cpu.get_flag(Flag::Z), false);

        cpu.change_register(InstructionRegister::A, -5);
        cpu.execute_aci(-5);
        assert_eq!(cpu.get_register(InstructionRegister::A), -10);
        assert_eq!(cpu.get_flag(Flag::S), true);

        cpu.change_register(InstructionRegister::A, -64);
        cpu.set_flag(Flag::C, true);
        cpu.execute_aci(64);
        assert_eq!(cpu.get_register(InstructionRegister::A), 1);
        assert_eq!(cpu.get_flag(Flag::C), true);

        cpu.change_register(InstructionRegister::A, 127);
        cpu.set_flag(Flag::C, false);
        cpu.execute_aci(127);
        assert_eq!(cpu.get_register(InstructionRegister::A), -2);
        assert_eq!(cpu.get_flag(Flag::C), false);

        cpu.set_flag(Flag::C, true);
        cpu.change_register(InstructionRegister::A, 0);
        cpu.execute_aci(4);
        assert_eq!(cpu.get_register(InstructionRegister::A), 5);

        cpu.set_flag(Flag::C, false);
        cpu.change_register(InstructionRegister::A, 0);
        cpu.execute_aci(4);
        assert_eq!(cpu.get_register(InstructionRegister::A), 4);
    }

    #[test]
    fn test_excute_sui() {
        let mut cpu = initialize_cpu();

        cpu.set_flag(Flag::Z, false);
        cpu.change_register(InstructionRegister::A, 5);
        cpu.execute_sui(5);
        assert_eq!(cpu.get_register(InstructionRegister::A), 0);
        assert_eq!(cpu.get_flag(Flag::Z), true);

        cpu.set_flag(Flag::Z, true);
        cpu.change_register(InstructionRegister::A, -5);
        cpu.execute_sui(8);
        assert_eq!(cpu.get_register(InstructionRegister::A), -13);
        assert_eq!(cpu.get_flag(Flag::Z), false);

        cpu.set_flag(Flag::S, false);
        cpu.change_register(InstructionRegister::A, 10);
        cpu.execute_sui(16);
        assert_eq!(cpu.get_register(InstructionRegister::A), -6);
        assert_eq!(cpu.get_flag(Flag::S), true);

        cpu.set_flag(Flag::S, true);
        cpu.change_register(InstructionRegister::A, 10);
        cpu.execute_sui(1);
        assert_eq!(cpu.get_register(InstructionRegister::A), 9);
        assert_eq!(cpu.get_flag(Flag::S), false);

        cpu.change_register(InstructionRegister::A, 127);
        cpu.execute_sui(-1);
        assert_eq!(cpu.get_register(InstructionRegister::A), -128);
        assert_eq!(cpu.get_flag(Flag::C), true);

        cpu.set_flag(Flag::C, true);
        cpu.change_register(InstructionRegister::A, 10);
        cpu.execute_sui(1);
        assert_eq!(cpu.get_register(InstructionRegister::A), 9);
        assert_eq!(cpu.get_flag(Flag::C), false);
    }

    #[test]
    fn test_execute_sub() {
        let mut cpu = initialize_cpu();
        cpu.change_register(InstructionRegister::A, 5);

        cpu.execute_sub(&InstructionRegister::A);
        assert_eq!(cpu.get_register(InstructionRegister::A), 0);
        assert_eq!(cpu.get_flag(Flag::Z), true);

        cpu.change_register(InstructionRegister::A, -5);
        cpu.execute_sub(&InstructionRegister::A);
        assert_eq!(cpu.get_register(InstructionRegister::A), 0);

        cpu.change_register(InstructionRegister::A, 127);
        cpu.change_register(InstructionRegister::B, -1);
        cpu.execute_sub(&InstructionRegister::B);
        assert_eq!(cpu.get_register(InstructionRegister::A), -128);
        assert_eq!(cpu.get_flag(Flag::C), true);

        cpu.change_register(InstructionRegister::A, -59);
        cpu.change_register(InstructionRegister::B, -98);
        cpu.set_flag(Flag::C, true);
        cpu.execute_sub(&InstructionRegister::B);
        assert_eq!(cpu.get_register(InstructionRegister::A), 39);
        assert_eq!(cpu.get_flag(Flag::C), false);

        cpu.change_register(InstructionRegister::A, 12);
        cpu.change_register(InstructionRegister::B, -15);
        cpu.set_flag(Flag::C, false);
        cpu.execute_sub(&InstructionRegister::B);
        assert_eq!(cpu.get_register(InstructionRegister::A), 27);
        assert_eq!(cpu.get_flag(Flag::C), true);
    }

    #[test]
    fn test_execute_inr() {
        let mut cpu = initialize_cpu();

        cpu.execute_inr(&InstructionRegister::A);
        assert_eq!(cpu.get_register(InstructionRegister::A), 1);

        cpu.change_register(InstructionRegister::A, -2);
        cpu.execute_inr(&InstructionRegister::A);
        assert_eq!(cpu.get_register(InstructionRegister::A), -1);
        assert_eq!(cpu.get_flag(Flag::S), true);
    }

    #[test]
    fn test_execute_dcr() {
        let mut cpu = initialize_cpu();
        cpu.change_register(InstructionRegister::A, 1);

        cpu.execute_dcr(&InstructionRegister::A);
        assert_eq!(cpu.get_register(InstructionRegister::A), 0);
        assert_eq!(cpu.get_flag(Flag::Z), true);

        cpu.change_register(InstructionRegister::A, -1);

        cpu.execute_dcr(&InstructionRegister::A);
        assert_eq!(cpu.get_register(InstructionRegister::A), -2);
        assert_eq!(cpu.get_flag(Flag::S), true);
    }

    #[test]
    fn test_execute_ana() {
        let mut cpu = initialize_cpu();
        cpu.change_register(InstructionRegister::A, -10);
        cpu.change_register(InstructionRegister::B, -10);

        cpu.execute_ana(&InstructionRegister::B);
        assert_eq!(cpu.get_register(InstructionRegister::A), -10);

        // -15 11110001
        // -10 11110110
        // ANA 11110000

        cpu.change_register(InstructionRegister::A, -15);
        cpu.execute_ana(&InstructionRegister::B);
        assert_eq!(cpu.get_register(InstructionRegister::A), -16);
    }

    #[test]
    fn test_execute_stc() {
        let mut cpu = initialize_cpu();

        cpu.execute_stc();
        assert_eq!(cpu.get_flag(Flag::C), true);
    }

    #[test]
    fn test_execute_cmc() {
        let mut cpu = initialize_cpu();

        cpu.execute_cmc();
        assert_eq!(cpu.get_flag(Flag::C), true);

        cpu.execute_cmc();
        assert_eq!(cpu.get_flag(Flag::C), false);
    }

    #[test]
    fn test_execute_cma() {
        let mut cpu = initialize_cpu();

        cpu.change_register(InstructionRegister::A, 74);
        cpu.execute_cma();
        assert_eq!(cpu.get_register(InstructionRegister::A), -75);

        cpu.change_register(InstructionRegister::A, -45);
        cpu.execute_cma();
        assert_eq!(cpu.get_register(InstructionRegister::A), 44);
    }

    #[test]
    fn test_execute_rlc() {
        let mut cpu = initialize_cpu();

        // negative with carry
        cpu.set_flag(Flag::C, false);
        cpu.change_register(InstructionRegister::A, -14);
        cpu.execute_rlc();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(InstructionRegister::A), -27);
        // negative without carry
        cpu.set_flag(Flag::C, false);
        cpu.change_register(InstructionRegister::A, -128);
        cpu.execute_rlc();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(InstructionRegister::A), 1);

        // positive
        cpu.set_flag(Flag::C, true);
        cpu.change_register(InstructionRegister::A, 24);
        cpu.execute_rlc();
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(InstructionRegister::A), 48);
    }

    #[test]
    fn test_execute_rrc() {
        let mut cpu = initialize_cpu();

        // negative without carry
        cpu.set_flag(Flag::C, true);
        cpu.change_register(InstructionRegister::A, -14);
        cpu.execute_rrc();
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(InstructionRegister::A), 121);

        // negative with carry
        cpu.set_flag(Flag::C, false);
        cpu.change_register(InstructionRegister::A, -13);
        cpu.execute_rrc();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(InstructionRegister::A), -7);

        // positive without carry
        cpu.set_flag(Flag::C, true);
        cpu.change_register(InstructionRegister::A, 12);
        cpu.execute_rrc();
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(InstructionRegister::A), 6);

        // positive with carry
        cpu.set_flag(Flag::C, false);
        cpu.change_register(InstructionRegister::A, 13);
        cpu.execute_rrc();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(InstructionRegister::A), -122);
    }

    #[test]
    fn test_execute_ral() {
        let mut cpu = initialize_cpu();

        // false -> true
        cpu.set_flag(Flag::C, false);
        cpu.change_register(InstructionRegister::A, -75);
        cpu.execute_ral();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(InstructionRegister::A), 106);

        // true -> true
        cpu.set_flag(Flag::C, true);
        cpu.change_register(InstructionRegister::A, -75);
        cpu.execute_ral();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(InstructionRegister::A), 107);

        // false -> false
        cpu.set_flag(Flag::C, false);
        cpu.change_register(InstructionRegister::A, 12);
        cpu.execute_ral();
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(InstructionRegister::A), 24);

        // true -> false
        cpu.set_flag(Flag::C, true);
        cpu.change_register(InstructionRegister::A, 12);
        cpu.execute_ral();
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(InstructionRegister::A), 25);
    }

    #[test]
    fn test_execute_rar() {
        let mut cpu = initialize_cpu();

        // true -> false
        cpu.set_flag(Flag::C, true);
        cpu.change_register(InstructionRegister::A, 106);
        cpu.execute_rar();
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(InstructionRegister::A), -75);

        // false -> false
        cpu.set_flag(Flag::C, false);
        cpu.change_register(InstructionRegister::A, 106);
        cpu.execute_rar();
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(InstructionRegister::A), 53);

        // false -> true
        cpu.set_flag(Flag::C, false);
        cpu.change_register(InstructionRegister::A, 53);
        cpu.execute_rar();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(InstructionRegister::A), 26);

        // true -> true
        cpu.set_flag(Flag::C, true);
        cpu.change_register(InstructionRegister::A, 53);
        cpu.execute_rar();
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_register(InstructionRegister::A), -102);
    }

    #[test]
    fn test_execute_ora() {
        let mut cpu = initialize_cpu();

        cpu.set_flag(Flag::C, true);
        cpu.change_register(InstructionRegister::A, 51);
        cpu.change_register(InstructionRegister::B, 15);
        cpu.execute_ora(&InstructionRegister::B);
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(InstructionRegister::A), 63);

        cpu.set_flag(Flag::C, false);
        cpu.change_register(InstructionRegister::A, -1);
        cpu.change_register(InstructionRegister::B, 0);
        cpu.execute_ora(&InstructionRegister::B);
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_register(InstructionRegister::A), -1);
    }

    #[test]
    fn test_execute_daa() {
        let mut cpu = initialize_cpu();

        // neither carry bit are set
        cpu.set_flag(Flag::A, true);
        cpu.change_register(InstructionRegister::A, -101);
        cpu.execute_daa();
        assert_eq!(cpu.get_register(InstructionRegister::A), 1);
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_flag(Flag::A), true);
    }

    #[test]
    fn test_execute_stax() {
        let mut cpu = initialize_cpu();

        cpu.change_register(InstructionRegister::A, 42);
        cpu.change_register(InstructionRegister::B, 123);
        cpu.change_register(InstructionRegister::C, 17);

        cpu.execute_stax(&InstructionRegisterPair::BC);
        assert_eq!(cpu.get_memory(31505), 42);
    }

    #[test]
    fn test_execute_ldax() {
        let mut cpu = initialize_cpu();

        cpu.change_register(InstructionRegister::D, -109);
        cpu.change_register(InstructionRegister::E, -117);
        cpu.set_memory(37771, 42);
        cpu.execute_ldax(&InstructionRegisterPair::DE);
        assert_eq!(cpu.get_register(InstructionRegister::A), 42);
    }

    #[test]
    fn test_execute_cmp() {
        let mut cpu = initialize_cpu();

        cpu.set_flag(Flag::C, true);
        cpu.set_flag(Flag::Z, true);
        cpu.change_register(InstructionRegister::A, 10);
        cpu.change_register(InstructionRegister::E, -5);
        cpu.execute_cmp(&InstructionRegister::E);
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_flag(Flag::Z), false);

        cpu.set_flag(Flag::C, false);
        cpu.set_flag(Flag::Z, true);
        cpu.change_register(InstructionRegister::A, 2);
        cpu.change_register(InstructionRegister::E, -5);
        cpu.execute_cmp(&InstructionRegister::E);
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_flag(Flag::Z), false);

        cpu.set_flag(Flag::C, true);
        cpu.set_flag(Flag::Z, true);
        cpu.change_register(InstructionRegister::A, -27);
        cpu.change_register(InstructionRegister::E, -5);
        cpu.execute_cmp(&InstructionRegister::E);
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_flag(Flag::Z), false);
    }

    #[test]
    fn test_execute_xra() {
        let mut cpu = initialize_cpu();

        cpu.set_flag(Flag::Z, false);
        cpu.change_register(InstructionRegister::A, 123);
        cpu.execute_xra(&InstructionRegister::A);
        assert_eq!(cpu.get_register(InstructionRegister::A), 0);
        assert_eq!(cpu.get_flag(Flag::Z), true);

        cpu.set_flag(Flag::Z, true);
        cpu.change_register(InstructionRegister::A, 92);
        cpu.change_register(InstructionRegister::B, 120);
        cpu.execute_xra(&InstructionRegister::B);
        assert_eq!(cpu.get_register(InstructionRegister::B), 36);
        assert_eq!(cpu.get_flag(Flag::Z), false);
    }

    #[test]
    fn test_execute_sbb() {
        let mut cpu = initialize_cpu();

        cpu.set_flag(Flag::Z, true);
        cpu.set_flag(Flag::C, true);
        cpu.change_register(InstructionRegister::A, 4);
        cpu.change_register(InstructionRegister::L, 2);
        cpu.execute_sbb(&InstructionRegister::L);
        assert_eq!(cpu.get_register(InstructionRegister::A), 1);
        assert_eq!(cpu.get_flag(Flag::Z), false);
        assert_eq!(cpu.get_flag(Flag::C), false);
    }

    #[test]
    fn test_execute_xchg() {
        let mut cpu = initialize_cpu();

        cpu.change_register(InstructionRegister::D, 51);
        cpu.change_register(InstructionRegister::E, 85);
        cpu.change_register(InstructionRegister::H, 0);
        cpu.change_register(InstructionRegister::L, -128);
        cpu.execute_xchg();

        assert_eq!(cpu.get_register(InstructionRegister::H), 51);
        assert_eq!(cpu.get_register(InstructionRegister::L), 85);
        assert_eq!(cpu.get_register(InstructionRegister::D), 0);
        assert_eq!(cpu.get_register(InstructionRegister::E), -128);
    }

    #[test]
    fn test_execute_sphl() {
        let mut cpu = initialize_cpu();

        cpu.change_register(InstructionRegister::H, 80);
        cpu.change_register(InstructionRegister::L, 108);
        cpu.execute_sphl();

        assert_eq!(cpu.get_stack_pointer(), 20588);
    }

    #[test]
    fn test_execute_xthl() {
        let mut cpu = initialize_cpu();

        cpu.set_stack_pointer(4269);
        cpu.change_register(InstructionRegister::H, 11);
        cpu.change_register(InstructionRegister::L, 60);
        cpu.set_memory(4269, -16);
        cpu.set_memory(4270, 13);
        cpu.execute_xthl();

        assert_eq!(cpu.get_register(InstructionRegister::H), 13);
        assert_eq!(cpu.get_register(InstructionRegister::L), -16);
        assert_eq!(cpu.get_memory(4269), 60);
        assert_eq!(cpu.get_memory(4270), 11);
    }

    #[test]
    fn test_execute_dcx() {
        let mut cpu = initialize_cpu();

        cpu.execute_dcx(&InstructionRegisterPair::BC);
        assert_eq!(cpu.get_register(InstructionRegister::B), -1);
        assert_eq!(cpu.get_register(InstructionRegister::C), -1);

        cpu.change_register(InstructionRegister::H, -104);
        cpu.change_register(InstructionRegister::L, 0);
        cpu.execute_dcx(&InstructionRegisterPair::HL);
        assert_eq!(cpu.get_register(InstructionRegister::H), -105);
        assert_eq!(cpu.get_register(InstructionRegister::L), -1);

        cpu.execute_dcx(&InstructionRegisterPair::SP);
        assert_eq!(cpu.get_stack_pointer(), 65535);
    }

    #[test]
    fn test_execute_inx() {
        let mut cpu = initialize_cpu();

        cpu.execute_inx(&InstructionRegisterPair::BC);
        assert_eq!(cpu.get_register(InstructionRegister::B), 0);
        assert_eq!(cpu.get_register(InstructionRegister::C), 1);

        cpu.change_register(InstructionRegister::D, 56);
        cpu.change_register(InstructionRegister::E, -1);
        cpu.execute_inx(&InstructionRegisterPair::DE);
        assert_eq!(cpu.get_register(InstructionRegister::D), 57);
        assert_eq!(cpu.get_register(InstructionRegister::E), 0);

        cpu.set_stack_pointer(65535);
        cpu.execute_inx(&InstructionRegisterPair::SP);
        assert_eq!(cpu.get_stack_pointer(), 0);
    }

    #[test]
    fn test_execute_dad() {
        let mut cpu = initialize_cpu();

        cpu.set_flag(Flag::C, true);
        cpu.change_register(InstructionRegister::B, 51);
        cpu.change_register(InstructionRegister::C, -97);
        cpu.change_register(InstructionRegister::H, -95);
        cpu.change_register(InstructionRegister::L, 123);
        cpu.execute_dad(&InstructionRegisterPair::BC);

        assert_eq!(cpu.get_register(InstructionRegister::H), -43);
        assert_eq!(cpu.get_register(InstructionRegister::L), 26);
        assert_eq!(cpu.get_flag(Flag::C), false);

        cpu.set_flag(Flag::C, false);
        cpu.set_stack_pointer(1);
        cpu.change_register(InstructionRegister::H, -1);
        cpu.change_register(InstructionRegister::L, -1);
        cpu.execute_dad(&InstructionRegisterPair::SP);

        assert_eq!(cpu.get_register(InstructionRegister::H), 0);
        assert_eq!(cpu.get_register(InstructionRegister::L), 0);
        assert_eq!(cpu.get_flag(Flag::C), true);
    }

    #[test]
    fn test_execute_push() {
        let mut cpu = initialize_cpu();

        cpu.change_register(InstructionRegister::D, -113);
        cpu.change_register(InstructionRegister::E, -99);
        cpu.set_stack_pointer(14892);
        cpu.execute_push(&InstructionRegisterPair::DE);

        assert_eq!(cpu.get_memory(14891), -113);
        assert_eq!(cpu.get_memory(14890), -99);
        assert_eq!(cpu.get_stack_pointer(), 14890);

        cpu.change_register(InstructionRegister::A, 31);
        cpu.set_flag(Flag::C, true);
        cpu.set_flag(Flag::Z, true);
        cpu.set_flag(Flag::P, true);
        cpu.set_flag(Flag::S, false);
        cpu.set_flag(Flag::A, false);
        cpu.set_stack_pointer(20522);
        cpu.execute_push(&InstructionRegisterPair::FA);

        assert_eq!(cpu.get_memory(20521), 31);
        assert_eq!(cpu.get_memory(20520), 71);
        assert_eq!(cpu.get_stack_pointer(), 20520);
    }

    #[test]
    fn test_execute_pop() {
        let mut cpu = initialize_cpu();

        cpu.set_memory(4665, 61);
        cpu.set_memory(4666, -109);
        cpu.set_stack_pointer(4665);
        cpu.execute_pop(&InstructionRegisterPair::HL);

        assert_eq!(cpu.get_register(InstructionRegister::L), 61);
        assert_eq!(cpu.get_register(InstructionRegister::H), -109);
        assert_eq!(cpu.get_stack_pointer(), 4667);

        cpu.set_memory(11264, -61);
        cpu.set_memory(11265, -1);
        cpu.set_stack_pointer(11264);
        cpu.execute_pop(&InstructionRegisterPair::FA);

        assert_eq!(cpu.get_register(InstructionRegister::A), -1);
        assert_eq!(cpu.get_flag(Flag::S), true);
        assert_eq!(cpu.get_flag(Flag::Z), true);
        assert_eq!(cpu.get_flag(Flag::A), false);
        assert_eq!(cpu.get_flag(Flag::P), false);
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_stack_pointer(), 11266);
    }

    #[test]
    fn test_execute_ori() {
        let mut cpu = initialize_cpu();

        cpu.change_register(InstructionRegister::A, -75);
        cpu.execute_ori(15);
        assert_eq!(cpu.get_register(InstructionRegister::A), -65);

        cpu.change_register(InstructionRegister::A, 0);
        cpu.set_flag(Flag::C, true);
        cpu.execute_ori(0);
        assert_eq!(cpu.get_register(InstructionRegister::A), 0);
        assert_eq!(cpu.get_flag(Flag::Z), true);
        assert_eq!(cpu.get_flag(Flag::C), false);
    }

    #[test]
    fn test_execute_xri() {
        let mut cpu = initialize_cpu();

        cpu.change_register(InstructionRegister::A, 59);
        cpu.execute_xri(-127);
        assert_eq!(cpu.get_register(InstructionRegister::A), -70);

        cpu.change_register(InstructionRegister::A, 1);
        cpu.set_flag(Flag::C, true);
        cpu.execute_xri(1);
        assert_eq!(cpu.get_register(InstructionRegister::A), 0);
        assert_eq!(cpu.get_flag(Flag::Z), true);
        assert_eq!(cpu.get_flag(Flag::C), false);
    }

    #[test]
    fn test_execute_ani() {
        let mut cpu = initialize_cpu();

        cpu.change_register(InstructionRegister::A, 58);
        cpu.execute_ani(15);
        assert_eq!(cpu.get_register(InstructionRegister::A), 10);

        cpu.change_register(InstructionRegister::A, -1);
        cpu.set_flag(Flag::C, true);
        cpu.execute_ani(0);
        assert_eq!(cpu.get_register(InstructionRegister::A), 0);
        assert_eq!(cpu.get_flag(Flag::Z), true);
        assert_eq!(cpu.get_flag(Flag::C), false);
    }

    #[test]
    fn test_execute_cpi() {
        let mut cpu = initialize_cpu();

        cpu.set_flag(Flag::C, true);
        cpu.set_flag(Flag::Z, true);
        cpu.change_register(InstructionRegister::A, 74);
        cpu.execute_cpi(64);
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_flag(Flag::Z), false);

        cpu.set_flag(Flag::C, true);
        cpu.set_flag(Flag::Z, true);
        cpu.change_register(InstructionRegister::A, 74);
        cpu.execute_cpi(-64);
        assert_eq!(cpu.get_flag(Flag::C), false);
        assert_eq!(cpu.get_flag(Flag::Z), false);
    }

    #[test]
    fn test_execute_sbi() {
        let mut cpu = initialize_cpu();

        cpu.change_register(InstructionRegister::A, 0);
        cpu.set_flag(Flag::C, false);
        cpu.set_flag(Flag::Z, true);
        cpu.execute_sbi(1);
        assert_eq!(cpu.get_register(InstructionRegister::A), -1);
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_flag(Flag::Z), false);

        cpu.change_register(InstructionRegister::A, 0);
        cpu.set_flag(Flag::C, true);
        cpu.set_flag(Flag::Z, true);
        cpu.execute_sbi(1);
        assert_eq!(cpu.get_register(InstructionRegister::A), -2);
        assert_eq!(cpu.get_flag(Flag::C), true);
        assert_eq!(cpu.get_flag(Flag::Z), false);
    }
    #[test]
    fn test_execute_lxi() {
        let mut cpu = initialize_cpu();

        cpu.execute_lxi(&InstructionRegisterPair::BC, 4080);
        assert_eq!(cpu.get_register(InstructionRegister::B), 15);
        assert_eq!(cpu.get_register(InstructionRegister::C), -16);

        cpu.execute_lxi(&InstructionRegisterPair::SP, 4080);
        assert_eq!(cpu.get_stack_pointer(), 4080);
    }

    #[test]
    fn test_execute_sta() {
        let mut cpu = initialize_cpu();

        cpu.change_register(InstructionRegister::A, 15);
        cpu.execute_sta(123);

        assert_eq!(cpu.get_memory(123), 15);
    }

    #[test]
    fn test_execute_lda() {
        let mut cpu = initialize_cpu();

        cpu.set_memory(42, 123);
        cpu.execute_lda(42);

        assert_eq!(cpu.get_memory(42), 123);
    }

    #[test]
    fn test_execute_shld() {
        let mut cpu = initialize_cpu();

        cpu.change_register(InstructionRegister::H, 70);
        cpu.change_register(InstructionRegister::L, 7);
        cpu.execute_shld(123);

        assert_eq!(cpu.get_memory(123), 7);
        assert_eq!(cpu.get_memory(124), 70);
    }

    #[test]
    fn test_execute_lhld() {
        let mut cpu = initialize_cpu();

        cpu.set_memory(300, 15);
        cpu.set_memory(301, 1);
        cpu.execute_lhld(300);

        assert_eq!(cpu.get_register(InstructionRegister::H), 1);
        assert_eq!(cpu.get_register(InstructionRegister::L), 15);
    }

    #[test]
    fn test_execute_pchl() {
        let mut cpu = initialize_cpu();

        cpu.change_register(InstructionRegister::H, 65);
        cpu.change_register(InstructionRegister::L, 62);
        cpu.execute_pchl();

        assert_eq!(cpu.get_program_counter(), 16702);
    }

    #[test]
    fn test_execute_jmp() {
        let mut cpu = initialize_cpu();

        cpu.set_program_counter(10);
        cpu.execute_jmp(1234);

        assert_eq!(cpu.get_program_counter(), 1234);
    }

    #[test]
    fn test_memory() {
        let mut cpu = initialize_cpu();

        cpu.set_memory(65535, 42);
        assert_eq!(cpu.get_memory(65535), 42);
    }

    #[test]
    fn test_flag_get_index() {
        assert_eq!(Flag::S.get_index(), 0);
        assert_eq!(Flag::Z.get_index(), 1);
        assert_eq!(Flag::A.get_index(), 3);
        assert_eq!(Flag::P.get_index(), 5);
        assert_eq!(Flag::C.get_index(), 7);
    }
}
