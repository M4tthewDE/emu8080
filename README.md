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
- [ ] SBB
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
- [ ] PUSH
- [ ] POP
- [ ] DAD
- [ ] INX
- [ ] DCX
- [ ] XCHG
- [ ] XTHL
- [ ] SPHL

## Immediate Instructions
- [ ] LXI
- [x] MVI
- [x] ADI
- [x] ACI
- [x] SUI
- [ ] SBI
- [ ] ANI
- [ ] XRI
- [ ] ORI
- [ ] CPI

## Direct Addressing Instructions
- [ ] STA
- [ ] LDA
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
