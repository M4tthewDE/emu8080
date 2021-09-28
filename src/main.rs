mod assembler;
mod cpu;

fn main() {
    let mut cpu = cpu::initialize_cpu();

    // set A to 12, to see effects of assembly code execution
    cpu.set_register(0, cpu::initialize_register(12));
    
    let assembler = assembler::Assembler::new("test.asm".to_owned(), "output".to_owned());

    assembler.assemble();
    let instructions = assembler.disassemble("output".to_owned());
    cpu.run(&instructions);
}
