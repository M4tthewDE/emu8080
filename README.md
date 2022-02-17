# emu8080

Emulation of an intel 8080 8-bit processor.

# Supported instructions (WIP)

Source: [intel 8080 Assembly Language
Programming Manual](http://dunfield.classiccmp.org/r/8080asm.pdf)

## Carry Bit Instructions
- [x] CMC
- [x] STC

## Single Register Instructions
- [x] INR
- [x] DCR
- [x] CMA
- [x] DAA

## Data Transfer Instructions
- [x] MOV
- [x] LDAX
- [x] STAX

## Register/Memory to Accumulator Instructions
- [x] ADD
- [x] ADC
- [x] SUB
- [x] SBB
- [x] ANA
- [x] XRA
- [x] ORA
- [x] CMP

## Rotate Accumulator Instructions
- [x] RLC
- [x] RRC
- [x] RAL
- [x] RAR

## Register Pair Instructions
- [x] PUSH
- [x] POP
- [x] DAD
- [x] INX
- [x] DCX
- [x] XCHG
- [x] XTHL
- [x] SPHL

## Immediate Instructions
- [x] LXI
- [x] MVI
- [x] ADI
- [x] ACI
- [x] SUI
- [x] SBI
- [x] ANI
- [x] XRI
- [x] ORI
- [x] CPI

## Direct Addressing Instructions
- [x] STA
- [x] LDA
- [ ] SHLD
- [ ] LHLD

## Jump Instructions
- [ ] PCHL
- [ ] JMP
- [ ] JC
- [ ] JNC
- [ ] JZ
- [ ] JNZ
- [ ] JM
- [ ] JP
- [ ] JPE
- [ ] JPO

## Halt Instruction
- [x] HLT
