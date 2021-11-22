; example assembly programm for tests 

TEST:   
        MVI A,00011100 ; 28
        MOV A,B
        ANA B
        ADD A
        SUB A
        INR A
        DCR A 
        ADI 10011001 ; -103

LABEL:
        STC
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

STOP:
        HLT
