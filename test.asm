; example assembly programm for tests 

TEST:   MVI A,00011100 ; 28
        MOV A,B
        ANA B
        ADD A
        SUB A
        INR A
        DCR A 
        ADI 10011001 ; -103

LABEL:  STC
        CMC
        CMA
        ADC C
        ACI 00001100 ; 12
        SUI 00001100 ; 12
        RLC
        RRC
        RAL
        RAR
        ORA B
        DAA
        STAX B
        LDAX D
        CMP B
        XRA B
        SBB B
        XCHG
        SPHL
        XTHL
        DCX B
        INX SP
        DAD B
        PUSH PSW
        POP PSW
        ORI 00001111
        XRI 00001111
        ANI 10000000
        CPI 00001111
        SBI 00000000
        LXI SP,0011000000111001
        STA 0000000000101010
        LDA 0000000000000000
        SHLD 0011000000111001
        LHLD 0000111110100000
        JMP TEST1
        JC TEST1
        ADD B

TEST1:  ADD A

STOP:   HLT

