//! Documentation database for x86 instructions, Irvine32 library, and registers

use std::collections::HashMap;
use std::sync::LazyLock;

/// A documentation entry
#[derive(Debug, Clone)]
pub struct DocEntry {
    pub name: &'static str,
    pub syntax: &'static str,
    pub description: &'static str,
    pub example: Option<&'static str>,
}

impl DocEntry {
    const fn new(
        name: &'static str,
        syntax: &'static str,
        description: &'static str,
        example: Option<&'static str>,
    ) -> Self {
        Self {
            name,
            syntax,
            description,
            example,
        }
    }
}

/// Get documentation for a symbol (instruction, register, or Irvine32 function)
pub fn get_documentation(symbol: &str) -> Option<&'static DocEntry> {
    let lower = symbol.to_lowercase();
    DOCS.get(lower.as_str()).copied()
}

static DOCS: LazyLock<HashMap<&'static str, &'static DocEntry>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // Instructions
    for doc in INSTRUCTION_DOCS.iter() {
        map.insert(doc.name, doc);
    }

    // Registers
    for doc in REGISTER_DOCS.iter() {
        map.insert(doc.name, doc);
    }

    // Irvine32 functions
    for doc in IRVINE32_DOCS.iter() {
        map.insert(doc.name, doc);
    }

    map
});

// ============ x86 Instructions ============

static INSTRUCTION_DOCS: &[DocEntry] = &[
    // Data Movement
    DocEntry::new("mov", "MOV dest, src", "Move data from source to destination. Both operands must be the same size. Cannot move memory to memory directly.", Some("mov eax, 10      ; immediate to register\nmov ebx, eax     ; register to register\nmov [var], eax   ; register to memory")),
    DocEntry::new("movsx", "MOVSX dest, src", "Move with sign extension. Copies smaller signed value to larger register, preserving the sign.", Some("movsx eax, al   ; sign-extend AL to EAX\nmovsx eax, ax   ; sign-extend AX to EAX\nmovsx eax, BYTE PTR [ebx]")),
    DocEntry::new("movzx", "MOVZX dest, src", "Move with zero extension. Copies smaller unsigned value to larger register, filling upper bits with zeros.", Some("movzx eax, al   ; zero-extend AL to EAX\nmovzx eax, BYTE PTR [ebx]")),
    DocEntry::new("lea", "LEA dest, src", "Load effective address. Calculates memory address without accessing memory. Useful for arithmetic.", Some("lea eax, [ebx+ecx*4]     ; address calc\nlea eax, [eax+eax*2]     ; eax = eax * 3\nlea eax, [ebx+10]        ; eax = ebx + 10")),
    DocEntry::new("xchg", "XCHG op1, op2", "Exchange values between two operands atomically. At least one operand must be a register.", Some("xchg eax, ebx    ; swap eax and ebx\nxchg al, [var]   ; swap with memory")),
    DocEntry::new("push", "PUSH src", "Push value onto stack. Decrements ESP by operand size and stores value at [ESP].", Some("push eax         ; push register\npush 100         ; push immediate\npush DWORD PTR [var]  ; push memory")),
    DocEntry::new("pop", "POP dest", "Pop value from stack. Loads value from [ESP] and increments ESP by operand size.", Some("pop eax          ; pop to register\npop DWORD PTR [var]  ; pop to memory")),
    DocEntry::new("pushad", "PUSHAD", "Push all 32-bit general-purpose registers onto stack in order: EAX, ECX, EDX, EBX, original ESP, EBP, ESI, EDI.", Some("pushad           ; save all registers\n; ... your code ...\npopad            ; restore all")),
    DocEntry::new("popad", "POPAD", "Pop all 32-bit general-purpose registers from stack in reverse order of PUSHAD. ESP value is discarded.", None),
    DocEntry::new("pushfd", "PUSHFD", "Push 32-bit EFLAGS register onto the stack.", Some("pushfd\npop eax  ; get flags into eax")),
    DocEntry::new("popfd", "POPFD", "Pop 32-bit value from stack into EFLAGS register.", Some("push eax\npopfd    ; set flags from eax")),
    DocEntry::new("lahf", "LAHF", "Load AH from lower 8 bits of FLAGS (SF, ZF, AF, PF, CF).", None),
    DocEntry::new("sahf", "SAHF", "Store AH into lower 8 bits of FLAGS register.", None),
    DocEntry::new("bswap", "BSWAP reg32", "Byte swap. Reverses byte order of 32-bit register (little-endian to big-endian).", Some("mov eax, 12345678h\nbswap eax  ; eax = 78563412h")),
    DocEntry::new("cmovz", "CMOVcc dest, src", "Conditional move if zero (ZF=1). Move only if condition is met.", Some("cmp eax, ebx\ncmovz ecx, edx  ; if equal, ecx=edx")),
    DocEntry::new("cmove", "CMOVcc dest, src", "Conditional move if equal (ZF=1). Same as CMOVZ.", Some("cmove eax, ebx")),
    DocEntry::new("cmovne", "CMOVcc dest, src", "Conditional move if not equal (ZF=0).", Some("cmovne eax, ebx")),
    DocEntry::new("cmovg", "CMOVcc dest, src", "Conditional move if greater (signed). ZF=0 and SF=OF.", Some("cmovg eax, ebx")),
    DocEntry::new("cmovl", "CMOVcc dest, src", "Conditional move if less (signed). SF!=OF.", Some("cmovl eax, ebx")),
    DocEntry::new("cmova", "CMOVcc dest, src", "Conditional move if above (unsigned). CF=0 and ZF=0.", Some("cmova eax, ebx")),
    DocEntry::new("cmovb", "CMOVcc dest, src", "Conditional move if below (unsigned). CF=1.", Some("cmovb eax, ebx")),

    // Arithmetic
    DocEntry::new("add", "ADD dest, src", "Add source to destination. Affects CF (carry), OF (overflow), SF (sign), ZF (zero), AF, PF flags.", Some("add eax, 5       ; eax = eax + 5\nadd eax, ebx     ; eax = eax + ebx\nadd [var], 10    ; memory += 10")),
    DocEntry::new("sub", "SUB dest, src", "Subtract source from destination. Sets flags same as ADD. CF set if borrow needed.", Some("sub eax, 5       ; eax = eax - 5\nsub eax, ebx     ; eax = eax - ebx")),
    DocEntry::new("mul", "MUL src", "Unsigned multiply. For 32-bit: EDX:EAX = EAX * src. CF/OF set if high half non-zero.", Some("mov eax, 100\nmov ebx, 200\nmul ebx          ; EDX:EAX = 20000")),
    DocEntry::new("imul", "IMUL [dest,] src [,imm]", "Signed multiply. Three forms: one-operand (like MUL), two-operand, three-operand with immediate.", Some("imul ebx         ; EDX:EAX = EAX * EBX\nimul eax, ebx    ; eax = eax * ebx\nimul eax, ebx, 5 ; eax = ebx * 5")),
    DocEntry::new("div", "DIV src", "Unsigned divide. EDX:EAX / src -> EAX=quotient, EDX=remainder. Clear EDX first for 32-bit division!", Some("xor edx, edx     ; clear high bits!\nmov eax, 100\nmov ebx, 7\ndiv ebx          ; eax=14, edx=2")),
    DocEntry::new("idiv", "IDIV src", "Signed divide. EDX:EAX / src -> EAX=quotient, EDX=remainder. Use CDQ to sign-extend EAX first!", Some("mov eax, -100\ncdq              ; sign-extend to EDX:EAX\nmov ebx, 7\nidiv ebx         ; eax=-14, edx=-2")),
    DocEntry::new("inc", "INC dest", "Increment by 1. Sets OF, SF, ZF, AF, PF but NOT CF. Use ADD if you need CF.", Some("inc eax          ; eax++\ninc DWORD PTR [var]  ; memory++")),
    DocEntry::new("dec", "DEC dest", "Decrement by 1. Sets OF, SF, ZF, AF, PF but NOT CF. Use SUB if you need CF.", Some("dec eax          ; eax--\ndec ecx\njnz loop         ; loop until zero")),
    DocEntry::new("neg", "NEG dest", "Two's complement negation (dest = 0 - dest). Sets CF=1 unless dest was 0.", Some("neg eax          ; eax = -eax\n; To get absolute value:\ntest eax, eax\njns positive\nneg eax")),
    DocEntry::new("adc", "ADC dest, src", "Add with carry. dest = dest + src + CF. Used for multi-precision arithmetic.", Some("; 64-bit add EDX:EAX + ECX:EBX\nadd eax, ebx     ; low 32 bits\nadc edx, ecx     ; high 32 + carry")),
    DocEntry::new("sbb", "SBB dest, src", "Subtract with borrow. dest = dest - src - CF. Used for multi-precision arithmetic.", Some("; 64-bit subtract\nsub eax, ebx     ; low 32 bits\nsbb edx, ecx     ; high 32 - borrow")),
    DocEntry::new("cwd", "CWD", "Convert Word to Doubleword. Sign-extends AX into DX:AX for 16-bit signed division.", None),
    DocEntry::new("cdq", "CDQ", "Convert Doubleword to Quadword. Sign-extends EAX into EDX:EAX. Required before IDIV!", Some("mov eax, -50\ncdq              ; EDX = FFFFFFFFh\nidiv ecx")),
    DocEntry::new("cwde", "CWDE", "Convert Word to Doubleword Extended. Sign-extends AX into EAX.", Some("mov ax, -100\ncwde             ; eax = FFFFFF9Ch")),
    DocEntry::new("cbw", "CBW", "Convert Byte to Word. Sign-extends AL into AX.", Some("mov al, -5\ncbw              ; ax = FFFBh")),

    // Logic
    DocEntry::new("and", "AND dest, src", "Bitwise AND. Clears CF and OF, sets SF/ZF/PF. Useful for masking bits and testing.", Some("and eax, 0FFh    ; keep low byte\nand eax, 0FFFFFFFEh  ; clear bit 0\nand al, 11011111b ; clear bit 5")),
    DocEntry::new("or", "OR dest, src", "Bitwise OR. Clears CF and OF, sets SF/ZF/PF. Used to set specific bits.", Some("or eax, 1        ; set bit 0\nor al, 20h       ; set bit 5\nor eax, eax      ; test if zero (sets ZF)")),
    DocEntry::new("xor", "XOR dest, src", "Bitwise XOR. Clears CF and OF. XOR reg,reg is fastest way to zero a register.", Some("xor eax, eax     ; eax = 0 (2 bytes)\nxor eax, ebx     ; toggle bits\nxor al, 20h      ; toggle bit 5 (case)")),
    DocEntry::new("not", "NOT dest", "Bitwise NOT (one's complement). Inverts all bits. Does NOT affect any flags.", Some("not eax          ; eax = ~eax\n; To flip sign: NOT then INC\nnot eax\ninc eax          ; same as NEG")),
    DocEntry::new("shl", "SHL dest, count", "Shift left logical. Multiplies by 2^count. CF = last bit shifted out. Count can be imm8 or CL.", Some("shl eax, 1       ; eax *= 2\nshl eax, 4       ; eax *= 16\nshl eax, cl      ; eax *= 2^cl")),
    DocEntry::new("shr", "SHR dest, count", "Shift right logical (unsigned). Divides by 2^count, zero-fills from left.", Some("shr eax, 1       ; eax /= 2 (unsigned)\nshr eax, cl      ; variable shift")),
    DocEntry::new("sal", "SAL dest, count", "Shift arithmetic left. Identical to SHL.", Some("sal eax, 2       ; same as shl eax, 2")),
    DocEntry::new("sar", "SAR dest, count", "Shift arithmetic right (signed). Preserves sign bit. Rounds toward negative infinity.", Some("mov eax, -8\nsar eax, 1       ; eax = -4 (not -3!)")),
    DocEntry::new("rol", "ROL dest, count", "Rotate left. Bits shifted out left side enter on right. CF = last bit rotated.", Some("rol eax, 8       ; rotate one byte\nrol al, 4        ; swap nibbles")),
    DocEntry::new("ror", "ROR dest, count", "Rotate right. Bits shifted out right side enter on left. CF = last bit rotated.", Some("ror eax, 8       ; rotate one byte right")),
    DocEntry::new("rcl", "RCL dest, count", "Rotate left through carry. CF becomes bit 0, bit 31 becomes new CF.", Some("rcl eax, 1       ; 33-bit rotate left")),
    DocEntry::new("rcr", "RCR dest, count", "Rotate right through carry. CF becomes bit 31, bit 0 becomes new CF.", Some("rcr eax, 1       ; 33-bit rotate right")),
    DocEntry::new("shld", "SHLD dest, src, count", "Double-precision shift left. Bits from src fill vacated positions in dest.", Some("shld eax, ebx, 4 ; shift 4 bits from ebx into eax")),
    DocEntry::new("shrd", "SHRD dest, src, count", "Double-precision shift right. Bits from src fill vacated positions in dest.", Some("shrd eax, ebx, 4")),

    // Bit manipulation
    DocEntry::new("bt", "BT dest, bit", "Bit test. Copies specified bit to CF. Does not modify dest.", Some("bt eax, 5        ; CF = bit 5 of eax\njc bit_is_set")),
    DocEntry::new("bts", "BTS dest, bit", "Bit test and set. Copies bit to CF, then sets the bit to 1.", Some("bts eax, 0       ; CF = old bit 0, then set it")),
    DocEntry::new("btr", "BTR dest, bit", "Bit test and reset. Copies bit to CF, then clears the bit to 0.", Some("btr eax, 7       ; CF = old bit 7, then clear it")),
    DocEntry::new("btc", "BTC dest, bit", "Bit test and complement. Copies bit to CF, then toggles the bit.", Some("btc eax, 3       ; CF = old bit 3, then flip it")),
    DocEntry::new("bsf", "BSF dest, src", "Bit scan forward. Find index of lowest set bit. ZF=1 if src=0.", Some("bsf eax, ebx     ; eax = index of lowest 1 bit")),
    DocEntry::new("bsr", "BSR dest, src", "Bit scan reverse. Find index of highest set bit. ZF=1 if src=0.", Some("bsr eax, ebx     ; eax = index of highest 1 bit")),
    DocEntry::new("setcc", "SETcc dest", "Set byte to 1 if condition true, 0 otherwise. dest must be 8-bit.", Some("cmp eax, ebx\nsetl al          ; al = 1 if eax < ebx\nsetz al          ; al = 1 if equal")),
    DocEntry::new("setz", "SETZ dest", "Set byte if zero (ZF=1).", Some("test eax, eax\nsetz al          ; al = 1 if eax == 0")),
    DocEntry::new("setnz", "SETNZ dest", "Set byte if not zero (ZF=0).", Some("setnz al")),
    DocEntry::new("setc", "SETC dest", "Set byte if carry (CF=1). Same as SETB.", Some("setc al")),
    DocEntry::new("setnc", "SETNC dest", "Set byte if no carry (CF=0). Same as SETAE.", Some("setnc al")),
    DocEntry::new("setg", "SETG dest", "Set byte if greater (signed).", Some("setg al")),
    DocEntry::new("setl", "SETL dest", "Set byte if less (signed).", Some("setl al")),
    DocEntry::new("seta", "SETA dest", "Set byte if above (unsigned).", Some("seta al")),
    DocEntry::new("setb", "SETB dest", "Set byte if below (unsigned).", Some("setb al")),

    // Comparison
    DocEntry::new("cmp", "CMP op1, op2", "Compare by subtracting (op1-op2). Sets CF, OF, SF, ZF, AF, PF but doesn't store result.", Some("cmp eax, 10      ; compare eax with 10\nje equal_label   ; jump if eax == 10\njl less_than     ; jump if eax < 10 (signed)")),
    DocEntry::new("test", "TEST op1, op2", "Bitwise AND without storing result. Clears CF/OF, sets SF/ZF/PF. Commonly used to check if zero.", Some("test eax, eax    ; is eax zero?\njz is_zero\ntest al, 1       ; is bit 0 set?\njnz bit_set")),

    // Control Flow - Unconditional
    DocEntry::new("jmp", "JMP target", "Unconditional jump. Can be short (-128 to +127), near (same segment), or far (different segment).", Some("jmp short next   ; 2-byte jump\njmp loop_start   ; near jump\njmp [eax]        ; indirect jump")),
    DocEntry::new("call", "CALL target", "Call procedure. Pushes return address (EIP) onto stack and jumps to target.", Some("call MyProc      ; direct call\ncall [eax]       ; indirect call\ncall [vtable+4]  ; virtual call")),
    DocEntry::new("ret", "RET [imm16]", "Return from procedure. Pops return address into EIP. Optional immediate removes stack parameters.", Some("ret              ; simple return\nret 8            ; return and pop 8 bytes\nret 12           ; stdcall with 3 DWORDs")),
    DocEntry::new("retn", "RETN [imm16]", "Near return. Same as RET in flat memory model.", Some("retn 8")),

    // Control Flow - Conditional (equality)
    DocEntry::new("je", "JE target", "Jump if equal (ZF=1). Use after CMP for equality test. Alias: JZ.", Some("cmp eax, 5\nje is_five       ; if eax == 5")),
    DocEntry::new("jne", "JNE target", "Jump if not equal (ZF=0). Use after CMP for inequality. Alias: JNZ.", Some("cmp eax, 0\njne not_zero     ; if eax != 0")),
    DocEntry::new("jz", "JZ target", "Jump if zero (ZF=1). Use after TEST/AND/OR/XOR to check for zero. Same as JE.", Some("test eax, eax\njz is_zero       ; if eax == 0")),
    DocEntry::new("jnz", "JNZ target", "Jump if not zero (ZF=0). Most common conditional jump. Same as JNE.", Some("dec ecx\njnz loop_start   ; loop while ecx != 0")),

    // Control Flow - Conditional (signed comparison)
    DocEntry::new("jg", "JG target", "Jump if greater (signed). True when ZF=0 AND SF=OF. Alias: JNLE.", Some("cmp eax, ebx\njg eax_bigger    ; if (signed)eax > ebx")),
    DocEntry::new("jge", "JGE target", "Jump if greater or equal (signed). True when SF=OF. Alias: JNL.", Some("cmp eax, 0\njge non_negative ; if (signed)eax >= 0")),
    DocEntry::new("jl", "JL target", "Jump if less (signed). True when SF!=OF. Alias: JNGE.", Some("cmp eax, 0\njl is_negative   ; if (signed)eax < 0")),
    DocEntry::new("jle", "JLE target", "Jump if less or equal (signed). True when ZF=1 OR SF!=OF. Alias: JNG.", Some("cmp eax, ebx\njle not_greater  ; if (signed)eax <= ebx")),

    // Control Flow - Conditional (unsigned comparison)
    DocEntry::new("ja", "JA target", "Jump if above (unsigned). True when CF=0 AND ZF=0. Alias: JNBE.", Some("cmp eax, ebx\nja eax_above     ; if (unsigned)eax > ebx")),
    DocEntry::new("jae", "JAE target", "Jump if above or equal (unsigned). True when CF=0. Alias: JNB, JNC.", Some("cmp al, 'A'\njae maybe_upper  ; if al >= 'A'")),
    DocEntry::new("jb", "JB target", "Jump if below (unsigned). True when CF=1. Alias: JNAE, JC.", Some("cmp eax, ebx\njb eax_below     ; if (unsigned)eax < ebx")),
    DocEntry::new("jbe", "JBE target", "Jump if below or equal (unsigned). True when CF=1 OR ZF=1. Alias: JNA.", Some("cmp al, '9'\njbe maybe_digit  ; if al <= '9'")),

    // Control Flow - Conditional (flags)
    DocEntry::new("jc", "JC target", "Jump if carry (CF=1). Use after ADD/SUB to detect overflow. Same as JB.", Some("add eax, ebx\njc overflow")),
    DocEntry::new("jnc", "JNC target", "Jump if no carry (CF=0). Same as JAE.", Some("sub eax, ebx\njnc no_borrow")),
    DocEntry::new("jo", "JO target", "Jump if overflow (OF=1). Signed overflow detected.", Some("add eax, ebx\njo signed_overflow")),
    DocEntry::new("jno", "JNO target", "Jump if no overflow (OF=0).", None),
    DocEntry::new("js", "JS target", "Jump if sign (SF=1). Result is negative.", Some("sub eax, ebx\njs went_negative")),
    DocEntry::new("jns", "JNS target", "Jump if no sign (SF=0). Result is non-negative.", Some("test eax, eax\njns is_positive_or_zero")),
    DocEntry::new("jp", "JP target", "Jump if parity (PF=1). Even number of 1 bits in low byte. Alias: JPE.", None),
    DocEntry::new("jnp", "JNP target", "Jump if no parity (PF=0). Odd number of 1 bits in low byte. Alias: JPO.", None),
    DocEntry::new("jcxz", "JCXZ target", "Jump if CX=0. Does not affect flags. Short jump only.", Some("jcxz skip_loop")),
    DocEntry::new("jecxz", "JECXZ target", "Jump if ECX=0. Does not affect flags. Short jump only.", Some("jecxz skip_loop")),

    // Loops
    DocEntry::new("loop", "LOOP target", "Decrement ECX and jump if ECX!=0. Does not affect flags. Short jump only.", Some("    mov ecx, 10\nL1: ; loop body here\n    loop L1      ; repeat 10 times")),
    DocEntry::new("loope", "LOOPE target", "Loop while equal. Dec ECX, jump if ECX!=0 AND ZF=1. Alias: LOOPZ.", Some("; Find first non-match\nrepe cmpsb\nloope search")),
    DocEntry::new("loopne", "LOOPNE target", "Loop while not equal. Dec ECX, jump if ECX!=0 AND ZF=0. Alias: LOOPNZ.", Some("; Find first match\nrepne scasb\nloopne search")),
    DocEntry::new("loopz", "LOOPZ target", "Loop while zero. Same as LOOPE.", None),
    DocEntry::new("loopnz", "LOOPNZ target", "Loop while not zero. Same as LOOPNE.", None),

    // Stack Frame
    DocEntry::new("enter", "ENTER imm16, imm8", "Create stack frame. imm16 = local var space, imm8 = nesting level (usually 0).", Some("enter 16, 0      ; 16 bytes locals\n; equivalent to:\n;   push ebp\n;   mov ebp, esp\n;   sub esp, 16")),
    DocEntry::new("leave", "LEAVE", "Destroy stack frame. Restores ESP and EBP.", Some("leave\nret\n; equivalent to:\n;   mov esp, ebp\n;   pop ebp")),

    // String Operations
    DocEntry::new("movsb", "MOVSB", "Move byte from [ESI] to [EDI]. Increments or decrements both pointers based on DF.", Some("cld              ; forward direction\nmov esi, OFFSET src\nmov edi, OFFSET dst\nmov ecx, 100\nrep movsb        ; copy 100 bytes")),
    DocEntry::new("movsw", "MOVSW", "Move word from [ESI] to [EDI]. Adjusts pointers by 2.", Some("rep movsw        ; copy ECX words")),
    DocEntry::new("movsd", "MOVSD", "Move dword from [ESI] to [EDI]. Adjusts pointers by 4. Fastest for aligned data.", Some("shr ecx, 2       ; bytes to dwords\nrep movsd        ; copy dwords")),
    DocEntry::new("cmpsb", "CMPSB", "Compare byte [ESI] with [EDI]. Sets flags, updates pointers.", Some("repe cmpsb       ; find first difference")),
    DocEntry::new("cmpsw", "CMPSW", "Compare word [ESI] with [EDI]. Sets flags, adjusts pointers by 2.", None),
    DocEntry::new("cmpsd", "CMPSD", "Compare dword [ESI] with [EDI]. Sets flags, adjusts pointers by 4.", None),
    DocEntry::new("scasb", "SCASB", "Scan string. Compare AL with [EDI], update EDI.", Some("mov edi, OFFSET str\nmov al, 0\nmov ecx, -1\nrepne scasb      ; find null terminator\nnot ecx\ndec ecx          ; ecx = string length")),
    DocEntry::new("scasw", "SCASW", "Scan string. Compare AX with [EDI], adjust EDI by 2.", None),
    DocEntry::new("scasd", "SCASD", "Scan string. Compare EAX with [EDI], adjust EDI by 4.", None),
    DocEntry::new("stosb", "STOSB", "Store AL to [EDI], update EDI.", Some("mov al, 0\nmov ecx, 100\nrep stosb        ; zero 100 bytes")),
    DocEntry::new("stosw", "STOSW", "Store AX to [EDI], adjust EDI by 2.", None),
    DocEntry::new("stosd", "STOSD", "Store EAX to [EDI], adjust EDI by 4.", Some("mov eax, -1\nrep stosd        ; fill with FFFFFFFFh")),
    DocEntry::new("lodsb", "LODSB", "Load byte from [ESI] into AL, update ESI.", Some("; Process string byte by byte\nL1: lodsb\n    test al, al\n    jz done\n    ; process al\n    jmp L1")),
    DocEntry::new("lodsw", "LODSW", "Load word from [ESI] into AX, adjust ESI by 2.", None),
    DocEntry::new("lodsd", "LODSD", "Load dword from [ESI] into EAX, adjust ESI by 4.", None),
    DocEntry::new("rep", "REP instruction", "Repeat string instruction ECX times. ECX decremented each iteration.", Some("rep movsb        ; copy ECX bytes\nrep stosb        ; fill ECX bytes")),
    DocEntry::new("repe", "REPE instruction", "Repeat while equal (ZF=1) AND ECX!=0. Also REPZ.", Some("repe cmpsb       ; compare until mismatch")),
    DocEntry::new("repne", "REPNE instruction", "Repeat while not equal (ZF=0) AND ECX!=0. Also REPNZ.", Some("repne scasb      ; scan until match")),
    DocEntry::new("repz", "REPZ instruction", "Repeat while zero. Same as REPE.", None),
    DocEntry::new("repnz", "REPNZ instruction", "Repeat while not zero. Same as REPNE.", None),

    // Flag Operations
    DocEntry::new("clc", "CLC", "Clear carry flag (CF=0). Useful before ADC chain or to indicate success.", Some("clc              ; clear carry\nret              ; return with CF=0 (success)")),
    DocEntry::new("stc", "STC", "Set carry flag (CF=1). Can indicate error return.", Some("stc              ; set carry\nret              ; return with CF=1 (error)")),
    DocEntry::new("cmc", "CMC", "Complement (toggle) carry flag. CF = NOT CF.", None),
    DocEntry::new("cld", "CLD", "Clear direction flag (DF=0). String operations auto-increment. ALWAYS call before string ops!", Some("cld              ; IMPORTANT!\nrep movsb        ; copy forward")),
    DocEntry::new("std", "STD", "Set direction flag (DF=1). String operations auto-decrement. Used for reverse copy.", Some("std              ; backward direction\n; copy from end to start\n; (for overlapping regions)")),
    DocEntry::new("cli", "CLI", "Clear interrupt flag (IF=0). Disable maskable interrupts. Privileged.", None),
    DocEntry::new("sti", "STI", "Set interrupt flag (IF=1). Enable maskable interrupts. Privileged.", None),
    DocEntry::new("pushf", "PUSHF", "Push 16-bit FLAGS onto stack.", None),
    DocEntry::new("popf", "POPF", "Pop 16-bit FLAGS from stack.", None),

    // Misc
    DocEntry::new("nop", "NOP", "No operation. Single-byte instruction. Used for padding, alignment, or timing.", Some("nop              ; 1 byte: 90h\n; Multi-byte NOPs exist for alignment")),
    DocEntry::new("hlt", "HLT", "Halt processor until next interrupt. Privileged instruction.", None),
    DocEntry::new("int", "INT imm8", "Software interrupt. Pushes FLAGS, CS, IP and jumps to interrupt handler.", Some("int 3            ; breakpoint (debug)\nint 21h          ; DOS services\nint 80h          ; Linux syscall (32-bit)")),
    DocEntry::new("into", "INTO", "Interrupt on overflow. Calls INT 4 if OF=1.", None),
    DocEntry::new("iret", "IRET", "Interrupt return. Pops IP, CS, FLAGS from stack.", None),
    DocEntry::new("xlat", "XLAT", "Table lookup. AL = [EBX + AL]. Translates byte using table.", Some("mov ebx, OFFSET table\nmov al, index\nxlat             ; al = table[index]")),
    DocEntry::new("xlatb", "XLATB", "Same as XLAT. Explicit byte form.", None),
    DocEntry::new("cpuid", "CPUID", "CPU identification. EAX=function, returns info in EAX,EBX,ECX,EDX.", Some("mov eax, 0\ncpuid            ; get vendor string")),
    DocEntry::new("rdtsc", "RDTSC", "Read time-stamp counter into EDX:EAX. Useful for timing.", Some("rdtsc\nmov [start], eax")),

    // MASM Directives
    DocEntry::new("invoke", "INVOKE proc [,args...]", "High-level procedure call. Pushes args right-to-left (stdcall), calls proc, cleans stack.", Some("invoke MessageBoxA, 0, ADDR msg, ADDR title, 0\ninvoke ExitProcess, 0")),
    DocEntry::new("proc", "name PROC [distance] [langtype] [visibility] [prologuearg] [USES regs] [,params]", "Define procedure with optional stack frame, saved registers, and parameters.", Some("MyFunc PROC USES ebx esi edi,\n    param1:DWORD,\n    param2:PTR BYTE\n    LOCAL var1:DWORD\n    ; function body\n    ret\nMyFunc ENDP")),
    DocEntry::new("endp", "name ENDP", "End procedure definition. Name must match PROC.", Some("MyProc ENDP")),
    DocEntry::new("proto", "name PROTO [distance] [langtype] [,params]", "Procedure prototype. Required for INVOKE. Declares parameter types.", Some("MyFunc PROTO, x:DWORD, y:DWORD\nWriteString PROTO")),
    DocEntry::new("local", "LOCAL varname:type [,varname:type]...", "Declare local variables in procedure. Allocates stack space.", Some("LOCAL buffer[256]:BYTE\nLOCAL count:DWORD, ptr:PTR BYTE")),
    DocEntry::new("uses", "USES reg [reg]...", "Save/restore registers automatically in procedure.", Some("MyProc PROC USES eax ebx ecx")),
    DocEntry::new("macro", "name MACRO [params]", "Define macro. Expanded inline at assembly time.", Some("mWriteStr MACRO string\n    push edx\n    mov edx, OFFSET string\n    call WriteString\n    pop edx\nENDM")),
    DocEntry::new("endm", "ENDM", "End macro definition.", None),
    DocEntry::new("exitm", "EXITM [value]", "Exit macro early, optionally returning a value.", None),
    DocEntry::new("equ", "name EQU expression", "Define symbolic constant. Can be numeric or text.", Some("CR EQU 13\nLF EQU 10\nMAX_SIZE EQU 1024")),
    DocEntry::new("textequ", "name TEXTEQU <text>", "Define text macro. Substituted as text.", Some("prompt TEXTEQU <\"Enter value: \">")),
    DocEntry::new("=", "name = expression", "Numeric assignment. Can be redefined (unlike EQU).", Some("count = 0\ncount = count + 1")),
    DocEntry::new("include", "INCLUDE filename", "Include source file at this point.", Some("INCLUDE Irvine32.inc\nINCLUDE macros.inc")),
    DocEntry::new("includelib", "INCLUDELIB filename", "Link with specified library.", Some("INCLUDELIB Irvine32.lib\nINCLUDELIB kernel32.lib\nINCLUDELIB user32.lib")),
    DocEntry::new(".data", ".DATA", "Begin initialized data segment. Variables with initial values.", Some(".data\nmyVar DWORD 100\nmyStr BYTE \"Hello\",0")),
    DocEntry::new(".data?", ".DATA?", "Begin uninitialized data segment. Variables without initial values (BSS).", Some(".data?\nbuffer BYTE 1024 DUP(?)")),
    DocEntry::new(".code", ".CODE [name]", "Begin code segment.", Some(".code\nmain PROC\n    ; code here\nmain ENDP")),
    DocEntry::new(".const", ".CONST", "Begin constant data segment. Read-only data.", Some(".const\nPI REAL8 3.14159265358979")),
    DocEntry::new(".stack", ".STACK [size]", "Define stack segment size. Default is 1024 bytes.", Some(".stack 4096")),
    DocEntry::new(".model", ".MODEL memmodel [,langtype] [,stacktype]", "Define memory model and calling convention.", Some(".model flat, stdcall")),
    DocEntry::new(".386", ".386 [P]", "Enable 80386 instruction set. P for privileged instructions.", Some(".386")),
    DocEntry::new(".486", ".486 [P]", "Enable 80486 instruction set.", None),
    DocEntry::new(".586", ".586 [P]", "Enable Pentium instruction set.", None),
    DocEntry::new(".686", ".686 [P]", "Enable Pentium Pro instruction set.", Some(".686P")),
    DocEntry::new("end", "END [startlabel]", "End of source file. Optional entry point label.", Some("END main")),
    DocEntry::new("public", "PUBLIC symbol [,symbol]...", "Make symbols visible to other modules.", Some("PUBLIC MyFunc, MyVar")),
    DocEntry::new("extern", "EXTERN name:type [,name:type]...", "Declare external symbol defined in another module.", Some("EXTERN printf:PROC\nEXTERN errno:DWORD")),
    DocEntry::new("extrn", "EXTRN name:type", "Same as EXTERN. Alternate spelling.", None),
    DocEntry::new("offset", "OFFSET expression", "Get address (offset) of variable or label.", Some("mov esi, OFFSET myArray\nlea esi, myArray  ; equivalent")),
    DocEntry::new("addr", "ADDR expression", "Address of. Used with INVOKE for arguments.", Some("invoke WriteString, ADDR myString")),
    DocEntry::new("ptr", "type PTR expression", "Pointer type override. Specifies operand size.", Some("mov BYTE PTR [eax], 0\nmov al, BYTE PTR myDword\ninc DWORD PTR [ebx]")),
    DocEntry::new("sizeof", "SIZEOF variable", "Size in bytes of variable or type.", Some("mov ecx, SIZEOF myArray")),
    DocEntry::new("lengthof", "LENGTHOF array", "Number of elements in array.", Some("mov ecx, LENGTHOF myArray")),
    DocEntry::new("type", "TYPE variable", "Size of one element of variable.", Some("mov eax, TYPE myArray  ; 4 for DWORD array")),
    DocEntry::new("dup", "count DUP (value)", "Duplicate initializer. Creates array or repeated values.", Some("buffer BYTE 100 DUP(0)      ; 100 zeros\narray DWORD 10 DUP(?)       ; 10 uninitialized\npattern BYTE 5 DUP(1,2,3)   ; 1,2,3,1,2,3,...")),

    // Data Types
    DocEntry::new("byte", "BYTE", "8-bit unsigned integer (0-255). Also DB directive.", Some("myByte BYTE 255\nstr BYTE \"Hello\",0")),
    DocEntry::new("sbyte", "SBYTE", "8-bit signed integer (-128 to 127).", Some("temp SBYTE -50")),
    DocEntry::new("word", "WORD", "16-bit unsigned integer. Also DW directive.", Some("myWord WORD 65535")),
    DocEntry::new("sword", "SWORD", "16-bit signed integer.", Some("val SWORD -1000")),
    DocEntry::new("dword", "DWORD", "32-bit unsigned integer. Also DD directive.", Some("myDword DWORD 12345678h")),
    DocEntry::new("sdword", "SDWORD", "32-bit signed integer.", Some("num SDWORD -100000")),
    DocEntry::new("qword", "QWORD", "64-bit integer. Also DQ directive.", Some("big QWORD 1234567890ABCDEFh")),
    DocEntry::new("real4", "REAL4", "32-bit IEEE single-precision float.", Some("pi REAL4 3.14159")),
    DocEntry::new("real8", "REAL8", "64-bit IEEE double-precision float.", Some("e REAL8 2.718281828")),
    DocEntry::new("real10", "REAL10", "80-bit extended precision float (FPU native).", Some("precise REAL10 3.14159265358979")),

    // Conditional Assembly
    DocEntry::new("if", "IF expression", "Conditional assembly. Assemble block if expression is true (non-zero).", Some("IF DEBUG\n    call DumpRegs\nENDIF")),
    DocEntry::new("ifdef", "IFDEF symbol", "Assemble if symbol is defined.", Some("IFDEF _DEBUG\n    ; debug code\nENDIF")),
    DocEntry::new("ifndef", "IFNDEF symbol", "Assemble if symbol is NOT defined.", Some("IFNDEF VERSION\nVERSION EQU 1\nENDIF")),
    DocEntry::new("else", "ELSE", "Alternative block for IF/IFDEF/IFNDEF.", Some("IF MODE EQ 1\n    ; mode 1\nELSE\n    ; other mode\nENDIF")),
    DocEntry::new("elseif", "ELSEIF expression", "Alternative condition.", Some("IF X EQ 1\n    ; x=1\nELSEIF X EQ 2\n    ; x=2\nENDIF")),
    DocEntry::new("endif", "ENDIF", "End conditional assembly block.", None),
];

// ============ Registers ============

static REGISTER_DOCS: &[DocEntry] = &[
    // 32-bit General Purpose
    DocEntry::new("eax", "EAX (Accumulator)", "32-bit accumulator. Return values, MUL/DIV operations. Caller-saved.", Some("; Return value from function\nmov eax, 0           ; return 0\n; MUL uses EAX implicitly\nmul ebx              ; EDX:EAX = EAX * EBX")),
    DocEntry::new("ebx", "EBX (Base)", "32-bit base register. General purpose. Callee-saved (must preserve).", Some("; Save before using\npush ebx\n; ... use ebx ...\npop ebx")),
    DocEntry::new("ecx", "ECX (Counter)", "32-bit counter. Loop counts, shift amounts, string op counts. Caller-saved.", Some("mov ecx, 10\nL1: ; loop body\n    loop L1          ; dec ecx, jnz")),
    DocEntry::new("edx", "EDX (Data)", "32-bit data register. I/O, MUL/DIV high bits. Caller-saved.", Some("; DIV uses EDX:EAX\nxor edx, edx         ; clear high bits\ndiv ebx              ; eax=quot, edx=rem")),
    DocEntry::new("esi", "ESI (Source Index)", "32-bit source index. Source pointer for string ops. Callee-saved.", Some("mov esi, OFFSET src\nlodsb                ; al = [esi], esi++")),
    DocEntry::new("edi", "EDI (Destination Index)", "32-bit destination index. Dest pointer for string ops. Callee-saved.", Some("mov edi, OFFSET dst\nstosb                ; [edi] = al, edi++")),
    DocEntry::new("esp", "ESP (Stack Pointer)", "32-bit stack pointer. Points to top of stack. Do NOT corrupt!", Some("; Push decrements ESP\npush eax             ; esp -= 4, [esp] = eax\n; Pop increments ESP\npop eax              ; eax = [esp], esp += 4")),
    DocEntry::new("ebp", "EBP (Base Pointer)", "32-bit base/frame pointer. Points to stack frame. Callee-saved.", Some("; Standard stack frame\npush ebp\nmov ebp, esp\nsub esp, 16          ; locals\n; [ebp+8] = 1st param\n; [ebp-4] = 1st local")),
    DocEntry::new("eip", "EIP (Instruction Pointer)", "32-bit instruction pointer. Address of next instruction. Cannot access directly.", Some("; Get EIP indirectly:\ncall next\nnext: pop eax        ; eax = address of 'next'")),

    // 16-bit General Purpose
    DocEntry::new("ax", "AX", "16-bit accumulator. Lower 16 bits of EAX. Modifying AX changes low 16 bits of EAX.", Some("mov ax, 1234h        ; EAX = xxxx1234h")),
    DocEntry::new("bx", "BX", "16-bit base. Lower 16 bits of EBX.", None),
    DocEntry::new("cx", "CX", "16-bit counter. Lower 16 bits of ECX.", None),
    DocEntry::new("dx", "DX", "16-bit data. Lower 16 bits of EDX.", None),
    DocEntry::new("si", "SI", "16-bit source index. Lower 16 bits of ESI.", None),
    DocEntry::new("di", "DI", "16-bit destination index. Lower 16 bits of EDI.", None),
    DocEntry::new("sp", "SP", "16-bit stack pointer. Lower 16 bits of ESP.", None),
    DocEntry::new("bp", "BP", "16-bit base pointer. Lower 16 bits of EBP.", None),

    // 8-bit General Purpose
    DocEntry::new("al", "AL", "Low 8 bits of AX/EAX. Used for byte operations, I/O, string ops.", Some("mov al, 'A'          ; character\nout 60h, al          ; I/O port")),
    DocEntry::new("ah", "AH", "High 8 bits of AX (bits 8-15). Note: modifying AL or AH affects AX.", Some("mov ax, 1234h\n; al = 34h, ah = 12h")),
    DocEntry::new("bl", "BL", "Low 8 bits of BX/EBX.", None),
    DocEntry::new("bh", "BH", "High 8 bits of BX (bits 8-15).", None),
    DocEntry::new("cl", "CL", "Low 8 bits of CX/ECX. Used for variable shift/rotate counts.", Some("mov cl, 5\nshl eax, cl          ; shift by cl bits")),
    DocEntry::new("ch", "CH", "High 8 bits of CX (bits 8-15).", None),
    DocEntry::new("dl", "DL", "Low 8 bits of DX/EDX.", None),
    DocEntry::new("dh", "DH", "High 8 bits of DX (bits 8-15).", None),

    // Segment Registers
    DocEntry::new("cs", "CS (Code Segment)", "Code segment selector. Points to code segment. Cannot MOV to CS directly.", None),
    DocEntry::new("ds", "DS (Data Segment)", "Data segment selector. Default segment for data access.", None),
    DocEntry::new("ss", "SS (Stack Segment)", "Stack segment selector. Used with ESP/EBP.", None),
    DocEntry::new("es", "ES (Extra Segment)", "Extra data segment. Used by string ops for destination.", Some("; String ops: DS:ESI -> ES:EDI\ncld\nrep movsb")),
    DocEntry::new("fs", "FS", "Additional segment. Windows: points to Thread Environment Block (TEB).", Some("; Windows: Get TEB\nmov eax, fs:[0]      ; SEH chain\nmov eax, fs:[18h]    ; TEB self-pointer")),
    DocEntry::new("gs", "GS", "Additional segment. Linux x64: thread-local storage.", None),

    // EFLAGS bits
    DocEntry::new("eflags", "EFLAGS", "32-bit flags register containing status, control, and system flags.", Some("; Common flags:\n; CF (bit 0)  - Carry\n; ZF (bit 6)  - Zero\n; SF (bit 7)  - Sign\n; OF (bit 11) - Overflow\n; DF (bit 10) - Direction")),
    DocEntry::new("cf", "CF (Carry Flag)", "Bit 0 of EFLAGS. Set on unsigned overflow/underflow. Used by ADC/SBB.", Some("add eax, ebx\njc overflow          ; CF=1 if carry out")),
    DocEntry::new("pf", "PF (Parity Flag)", "Bit 2 of EFLAGS. Set if low byte has even number of 1 bits.", None),
    DocEntry::new("af", "AF (Auxiliary Flag)", "Bit 4 of EFLAGS. BCD arithmetic carry from bit 3 to 4.", None),
    DocEntry::new("zf", "ZF (Zero Flag)", "Bit 6 of EFLAGS. Set if result is zero. Most common flag tested.", Some("cmp eax, ebx\nje equal             ; ZF=1 if eax==ebx\ntest eax, eax\njz is_zero           ; ZF=1 if eax==0")),
    DocEntry::new("sf", "SF (Sign Flag)", "Bit 7 of EFLAGS. Set if result is negative (MSB=1).", Some("test eax, eax\njs negative          ; SF=1 if eax<0")),
    DocEntry::new("df", "DF (Direction Flag)", "Bit 10 of EFLAGS. String operation direction. 0=forward, 1=backward.", Some("cld                  ; DF=0, forward\nstd                  ; DF=1, backward")),
    DocEntry::new("of", "OF (Overflow Flag)", "Bit 11 of EFLAGS. Set on signed overflow.", Some("add eax, ebx\njo overflow          ; OF=1 if signed overflow")),

    // FPU Registers
    DocEntry::new("st", "ST(0)", "Top of FPU register stack. 80-bit floating point.", Some("fld REAL8 PTR [x]    ; push x onto FPU stack\nfadd ST(0), ST(1)    ; ST(0) += ST(1)")),
    DocEntry::new("st0", "ST(0)", "FPU stack top. Same as ST.", None),
    DocEntry::new("st1", "ST(1)", "FPU stack register 1 (second from top).", None),
    DocEntry::new("st2", "ST(2)", "FPU stack register 2.", None),
    DocEntry::new("st3", "ST(3)", "FPU stack register 3.", None),
    DocEntry::new("st4", "ST(4)", "FPU stack register 4.", None),
    DocEntry::new("st5", "ST(5)", "FPU stack register 5.", None),
    DocEntry::new("st6", "ST(6)", "FPU stack register 6.", None),
    DocEntry::new("st7", "ST(7)", "FPU stack register 7 (bottom).", None),

    // SSE Registers
    DocEntry::new("xmm0", "XMM0", "128-bit SSE register. 4 floats or 2 doubles. Used for float returns.", Some("movss xmm0, [val]    ; load single float\naddps xmm0, xmm1     ; add 4 floats")),
    DocEntry::new("xmm1", "XMM1", "128-bit SSE register.", None),
    DocEntry::new("xmm2", "XMM2", "128-bit SSE register.", None),
    DocEntry::new("xmm3", "XMM3", "128-bit SSE register.", None),
    DocEntry::new("xmm4", "XMM4", "128-bit SSE register.", None),
    DocEntry::new("xmm5", "XMM5", "128-bit SSE register.", None),
    DocEntry::new("xmm6", "XMM6", "128-bit SSE register.", None),
    DocEntry::new("xmm7", "XMM7", "128-bit SSE register.", None),

    // 64-bit registers (for reference)
    DocEntry::new("rax", "RAX", "64-bit accumulator (x64 mode). Lower 32 bits is EAX.", None),
    DocEntry::new("rbx", "RBX", "64-bit base register (x64 mode).", None),
    DocEntry::new("rcx", "RCX", "64-bit counter (x64 mode). First integer arg in Win64 ABI.", None),
    DocEntry::new("rdx", "RDX", "64-bit data register (x64 mode). Second integer arg in Win64 ABI.", None),
    DocEntry::new("rsi", "RSI", "64-bit source index (x64 mode).", None),
    DocEntry::new("rdi", "RDI", "64-bit destination index (x64 mode). First arg in System V ABI.", None),
    DocEntry::new("rsp", "RSP", "64-bit stack pointer (x64 mode).", None),
    DocEntry::new("rbp", "RBP", "64-bit base pointer (x64 mode).", None),
    DocEntry::new("r8", "R8", "64-bit general purpose register (x64 only).", None),
    DocEntry::new("r9", "R9", "64-bit general purpose register (x64 only).", None),
    DocEntry::new("r10", "R10", "64-bit general purpose register (x64 only).", None),
    DocEntry::new("r11", "R11", "64-bit general purpose register (x64 only).", None),
    DocEntry::new("r12", "R12", "64-bit general purpose register (x64 only). Callee-saved.", None),
    DocEntry::new("r13", "R13", "64-bit general purpose register (x64 only). Callee-saved.", None),
    DocEntry::new("r14", "R14", "64-bit general purpose register (x64 only). Callee-saved.", None),
    DocEntry::new("r15", "R15", "64-bit general purpose register (x64 only). Callee-saved.", None),
];

// ============ Irvine32 Library ============

static IRVINE32_DOCS: &[DocEntry] = &[
    // Console Output
    DocEntry::new("writestring", "WriteString", "Display null-terminated string at current cursor position. Receives: EDX = OFFSET of string.", Some(".data\nmsg BYTE \"Hello, World!\",0\n.code\nmov edx, OFFSET msg\ncall WriteString")),
    DocEntry::new("writechar", "WriteChar", "Write single character to console. Receives: AL = character to display.", Some("mov al, 'A'\ncall WriteChar\nmov al, 10       ; newline\ncall WriteChar")),
    DocEntry::new("writedec", "WriteDec", "Write unsigned 32-bit integer as decimal. Receives: EAX = value.", Some("mov eax, 12345\ncall WriteDec    ; displays: 12345")),
    DocEntry::new("writeint", "WriteInt", "Write signed 32-bit integer as decimal with sign. Receives: EAX = value.", Some("mov eax, -42\ncall WriteInt    ; displays: -42")),
    DocEntry::new("writehex", "WriteHex", "Write unsigned 32-bit integer as 8-digit hex. Receives: EAX = value.", Some("mov eax, 255\ncall WriteHex    ; displays: 000000FF")),
    DocEntry::new("writehexb", "WriteHexB", "Write unsigned 8-bit integer as 2-digit hex. Receives: AL = value.", Some("mov al, 255\ncall WriteHexB   ; displays: FF")),
    DocEntry::new("writebin", "WriteBin", "Write unsigned 32-bit integer as 32-digit binary. Receives: EAX = value.", Some("mov eax, 255\ncall WriteBin    ; displays: 00000000000000000000000011111111")),
    DocEntry::new("writebinb", "WriteBinB", "Write unsigned 8-bit integer as 8-digit binary. Receives: AL = value.", Some("mov al, 255\ncall WriteBinB   ; displays: 11111111")),
    DocEntry::new("crlf", "Crlf", "Write carriage return + line feed (newline) to console. No parameters.", Some("call Crlf        ; move to next line")),

    // Console Input
    DocEntry::new("readstring", "ReadString", "Read string from keyboard. Receives: EDX=buffer OFFSET, ECX=max chars+1. Returns: EAX=chars read (excluding null).", Some(".data\nbuffer BYTE 81 DUP(?)\n.code\nmov edx, OFFSET buffer\nmov ecx, SIZEOF buffer\ncall ReadString\n; EAX = length of input")),
    DocEntry::new("readchar", "ReadChar", "Read single character from keyboard (waits for input). Returns: AL = character.", Some("call ReadChar    ; wait for key\ncmp al, 'y'\nje yes_pressed")),
    DocEntry::new("readdec", "ReadDec", "Read unsigned decimal integer from keyboard. Returns: EAX = value. CF=1 if invalid.", Some("call ReadDec\njc invalid       ; bad input?\nmov myVar, eax")),
    DocEntry::new("readint", "ReadInt", "Read signed decimal integer from keyboard. Returns: EAX = value. OF=1 if overflow.", Some("call ReadInt\njo overflow      ; too big?\nmov myVar, eax")),
    DocEntry::new("readhex", "ReadHex", "Read hexadecimal integer from keyboard. Returns: EAX = value.", Some("call ReadHex     ; enter: FF\n; EAX = 255")),
    DocEntry::new("readkey", "ReadKey", "Check for keyboard input (non-blocking). Returns: ZF=1 if no key, ZF=0 if key pressed. AL=ASCII, AH=scan code.", Some("L1: call ReadKey\n    jz L1        ; loop until key\n    cmp al, 27   ; ESC?\n    je quit")),

    // Screen Control
    DocEntry::new("clrscr", "Clrscr", "Clear console screen and reset cursor to (0,0).", Some("call Clrscr")),
    DocEntry::new("gotoxy", "Gotoxy", "Set cursor position. Receives: DH=row (0-based), DL=column (0-based).", Some("mov dh, 10       ; row 10\nmov dl, 20       ; column 20\ncall Gotoxy\n; cursor now at (20,10)")),
    DocEntry::new("getmaxxy", "GetMaxXY", "Get console size. Returns: AX=rows, DX=columns.", Some("call GetMaxXY\nmov rows, ax\nmov cols, dx")),
    DocEntry::new("gettextcolor", "GetTextColor", "Get current text color attribute. Returns: AL = color (foreground + background*16).", Some("call GetTextColor\nmov savedColor, al")),
    DocEntry::new("settextcolor", "SetTextColor", "Set text foreground and background colors. Receives: EAX = color attribute.", Some("; Colors: black=0, blue=1, green=2,\n; cyan=3, red=4, magenta=5, brown=6,\n; lightGray=7, gray=8, lightBlue=9,\n; lightGreen=10, lightCyan=11,\n; lightRed=12, lightMagenta=13,\n; yellow=14, white=15\nmov eax, yellow + (blue*16)\ncall SetTextColor")),

    // Timing and Messages
    DocEntry::new("waitmsg", "WaitMsg", "Display 'Press any key to continue...' and wait for keypress.", Some("call WaitMsg")),
    DocEntry::new("delay", "Delay", "Pause execution. Receives: EAX = milliseconds to wait.", Some("mov eax, 1000    ; 1 second\ncall Delay\nmov eax, 500     ; half second\ncall Delay")),
    DocEntry::new("getticks", "GetMseconds", "Get milliseconds elapsed since midnight. Returns: EAX = count.", Some("call GetMseconds\nmov startTime, eax\n; ... do work ...\ncall GetMseconds\nsub eax, startTime  ; elapsed time")),

    // Random Numbers
    DocEntry::new("randomize", "Randomize", "Seed random number generator with current time. Call once at program start.", Some("call Randomize   ; seed RNG\n; now Random32/RandomRange work")),
    DocEntry::new("random32", "Random32", "Generate pseudo-random 32-bit unsigned integer. Returns: EAX = random value.", Some("call Random32\nmov myRandom, eax")),
    DocEntry::new("randomrange", "RandomRange", "Generate random integer in range [0, n-1]. Receives: EAX = n. Returns: EAX = random.", Some("mov eax, 100\ncall RandomRange ; EAX = 0-99\nmov eax, 6\ncall RandomRange ; EAX = 0-5 (dice-1)")),

    // Debugging
    DocEntry::new("dumpregs", "DumpRegs", "Display all general-purpose registers and flags. Invaluable for debugging!", Some("call DumpRegs\n; Shows EAX, EBX, ECX, EDX,\n;       ESI, EDI, EBP, ESP,\n;       EIP, EFL (flags)")),
    DocEntry::new("dumpmem", "DumpMem", "Display memory as hex dump. Receives: ESI=OFFSET, ECX=count, EBX=TYPE (1/2/4).", Some("mov esi, OFFSET myArray\nmov ecx, LENGTHOF myArray\nmov ebx, TYPE myArray\ncall DumpMem")),
    DocEntry::new("dumpstack", "DumpStack", "Display stack contents.", Some("call DumpStack")),

    // String Operations
    DocEntry::new("str_length", "Str_length", "Get null-terminated string length. Receives: EDX=string OFFSET. Returns: EAX=length.", Some("mov edx, OFFSET myStr\ncall Str_length\nmov len, eax")),
    DocEntry::new("str_copy", "Str_copy", "Copy string. Receives: ESI=source OFFSET, EDI=destination OFFSET.", Some("mov esi, OFFSET source\nmov edi, OFFSET dest\ncall Str_copy")),
    DocEntry::new("str_compare", "Str_compare", "Compare strings (case-sensitive). Receives: ESI=str1, EDI=str2. Returns: ZF, CF like CMP.", Some("mov esi, OFFSET str1\nmov edi, OFFSET str2\ncall Str_compare\nje strings_equal\nja str1_greater")),
    DocEntry::new("str_ucase", "Str_ucase", "Convert string to uppercase in-place. Receives: EDX=string OFFSET.", Some("mov edx, OFFSET myStr\ncall Str_ucase")),
    DocEntry::new("str_trim", "Str_trim", "Remove trailing spaces from string. Receives: ESI=string OFFSET.", Some("mov esi, OFFSET myStr\ncall Str_trim")),

    // File I/O
    DocEntry::new("createoutputfile", "CreateOutputFile", "Create new file for writing. Receives: EDX=filename OFFSET. Returns: EAX=handle (-1 if error).", Some("mov edx, OFFSET filename\ncall CreateOutputFile\ncmp eax, -1\nje error\nmov fileHandle, eax")),
    DocEntry::new("openinputfile", "OpenInputFile", "Open existing file for reading. Receives: EDX=filename OFFSET. Returns: EAX=handle (-1 if error).", Some("mov edx, OFFSET filename\ncall OpenInputFile\ncmp eax, -1\nje file_not_found\nmov fileHandle, eax")),
    DocEntry::new("readfromfile", "ReadFromFile", "Read bytes from file. Receives: EAX=handle, EDX=buffer, ECX=max bytes. Returns: EAX=bytes read.", Some("mov eax, fileHandle\nmov edx, OFFSET buffer\nmov ecx, SIZEOF buffer\ncall ReadFromFile\nmov bytesRead, eax")),
    DocEntry::new("writetofile", "WriteToFile", "Write bytes to file. Receives: EAX=handle, EDX=buffer, ECX=count. Returns: EAX=bytes written.", Some("mov eax, fileHandle\nmov edx, OFFSET buffer\nmov ecx, bufferLen\ncall WriteToFile")),
    DocEntry::new("closefile", "CloseFile", "Close file handle. Receives: EAX=handle.", Some("mov eax, fileHandle\ncall CloseFile")),

    // Type Conversion
    DocEntry::new("parsedecimal32", "ParseDecimal32", "Convert decimal string to integer. Receives: EDX=string, ECX=length. Returns: EAX=value.", Some("mov edx, OFFSET numStr\nmov ecx, LENGTHOF numStr - 1\ncall ParseDecimal32")),
    DocEntry::new("parseinteger32", "ParseInteger32", "Convert signed decimal string to integer. Receives: EDX=string, ECX=length. Returns: EAX=value.", Some("mov edx, OFFSET numStr\ncall ParseInteger32")),
    DocEntry::new("writestackframe", "WriteStackFrame", "Display procedure's stack frame (parameters and locals).", Some("call WriteStackFrame")),
    DocEntry::new("writestackframeinfo", "WriteStackFrameInfo", "Display detailed stack frame info.", Some("call WriteStackFrameInfo")),

    // Special
    DocEntry::new("isdigit", "IsDigit", "Check if character is digit '0'-'9'. Receives: AL=char. Returns: ZF=1 if digit.", Some("mov al, char\ncall IsDigit\njz is_digit")),
    DocEntry::new("msgbox", "MsgBox", "Display Windows message box. Receives: EDX=message, EBX=title.", Some("mov edx, OFFSET message\nmov ebx, OFFSET title\ncall MsgBox")),
    DocEntry::new("msgboxask", "MsgBoxAsk", "Display Yes/No message box. Returns: EAX=6 (Yes) or 7 (No).", Some("mov edx, OFFSET question\nmov ebx, OFFSET title\ncall MsgBoxAsk\ncmp eax, 6\nje user_said_yes")),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_instruction_doc() {
        let doc = get_documentation("mov").unwrap();
        assert_eq!(doc.name, "mov");
        assert!(doc.description.contains("Move"));
    }

    #[test]
    fn test_get_register_doc() {
        let doc = get_documentation("eax").unwrap();
        assert_eq!(doc.name, "eax");
        assert!(doc.description.contains("accumulator"));
    }

    #[test]
    fn test_get_irvine_doc() {
        let doc = get_documentation("WriteString").unwrap();
        assert!(doc.description.contains("string"));
    }

    #[test]
    fn test_case_insensitive() {
        assert!(get_documentation("MOV").is_some());
        assert!(get_documentation("Mov").is_some());
        assert!(get_documentation("EAX").is_some());
    }
}
