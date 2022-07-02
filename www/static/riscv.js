
function isJumpImmediate(op){
    var jumps = ["jal", "jalr"];
    return jumps.some(x => x == op);
}

function isBranch(op){
    var branches = ["beq", "bne", "blt", "bge", "bltu", "bgeu"];
    return branches.some(x => x == op);
}

/* Memory is implemented using a hashmap of addresses to 8-bit unsigned values */
class Memory {

    constructor() {
        this.memory = {};
    }

    getMem(address) {
        address = address >>> 0;
        if (this.memory[address] === undefined) {
            return 0x00;
        }
        return this.memory[address] >>> 0;
    }

    setMem(address, value) {
        address = address >>> 0;
        value = value & 0xff; // truncates to 8-bit if needed
        this.memory[address] = value;
    }

    isValidAddress(address) {
        address = address >>> 0;
        return 0 <= address && address <= 0x7fffffff;
    }

}




/** Everything above this line is fully converted **/
class Program {

    constructor(instructions) {
        this.errors = [];
        this.pc = 0x0;
        this.line = 0;
        this.registers = new Int32Array(32);
        this.memory = new Memory();
        this.insns = instructions || ""; // if instructions is undefined, program is empty
        this.insns = this.insns.split('\n').map(function(insn) {
            return insn.trim();
        });
        this.labels = {};
        this.generateLabels();
        this.linkLabels();
        this.delaySlotInsnsPC = [];
        this.TOKEN_TYPE_REG = 0;
        this.TOKEN_TYPE_IMM = 1;
        this.numCycles = 0;
    }



     /** Goes through all the lines and finds the labels and associates pc addresses to them */
     generateLabels() {
        var filteredInstructions = [];
        var filteredIndex = 0;
        for (var i = 0; i < this.insns.length; ++i) {
            var lineNo = i + 1;
            var insn = this.insns[i];
            if (insn.indexOf('#') != -1) { // remove everything after a comment
                insn = insn.substring(0, insn.indexOf('#')).trim();
            }
            if (insn.charAt(insn.length-1) == ':') { // encounter a label which ends with a colon
                var label = insn.substring(0, insn.length-1);
                if (this.labels[label] !== undefined) {
                    this.pushError("Found multiple instances of label: " + label + " [line " + lineNo + "]");
                }
                if (label.charAt(0) >= '0' && label.charAt(0) <= '9') {
                    this.pushError("Label name cannot start with a number: " + label + " [line " + lineNo + "]");
                    continue;
                }
                this.labels[label] = filteredIndex; // make label point to the line after it (also zero-index -> one-index)
            }
            else if (insn != '') { // ignore empty/comment lines
                filteredInstructions.push([insn, lineNo, insn]); // push instruction and line number for debugging purposes
                filteredIndex++;
            }
        }
        this.insns = filteredInstructions;
    }

    /** Converts labels to memory locations */
    linkLabels() {
        for (var i = 0; i < this.insns.length; ++i) {
            var insn = this.insns[i][0];
            var lineNo = this.insns[i][1];
            if (insn.indexOf(' ') != -1) { // ignore changing labels of bad instructions
                var op = insn.substring(0, insn.indexOf(' ')).trim().toLowerCase();
                var tokens = insn.substring(insn.indexOf(' '), insn.length).split(',');
                var label = tokens[tokens.length-1].trim(); // label comes at the very end
                if (isJumpImmediate(op)) {
                    /* TODO: CHECK IF RISC JUMPS TO SHIFTED AMOUNT*/
                    if (this.labels[label] !== undefined) {
                        // tokens[tokens.length-1] = (this.labels[label]) << 2; // absolute jump to the label location
                        // tokens[tokens.length-1] = (this.labels[label] - (i + 1)) << 2; // branch offset no longer relative to delay slot instruction
                        tokens[tokens.length-1] = (this.labels[label] - i) << 2; // relative jump to the label location
                    }
                    else {
                        if (isNaN(parseInt(label))) {
                            this.pushError("Could not find label: " + label + " [line " + lineNo + "]");
                            tokens[tokens.length-1] = 0x2ffffff << 2; // most likely a label issue, so we want it to jump very far to the end
                        }
                    }
                }
                else if (isBranch(op)) {
                    /* TODO: REFACTOR BECAUSE DELAY SLOT CONSIDERATION */

                    if (this.labels[label] !== undefined) {
                        // tokens[tokens.length-1] = (this.labels[label] - (i + 1)) << 2; // branch offset relative to delay slot instruction
                        // tokens[tokens.length-1] = (this.labels[label]) << 2; // absolute jump to the label location
                        tokens[tokens.length-1] = (this.labels[label] - i) << 2; // relative jump to the label location
                    }
                    else {
                        if (isNaN(parseInt(label))) {
                            this.pushError("Could not find label: " + label + " [line " + lineNo + "]");
                            tokens[tokens.length-1] = 0x7fff << 2; // most likely a label issue, so we want it to branch very far to the end
                        }
                    }
                }
                this.insns[i][0] = op + " " + tokens.join(', '); // instruction with labels replaced
            }
        }
    }

    getRegisters() {
        return this.registers;
    }

    getMemory() {
        return this.memory;
    }

    getErrors() {
        return this.errors;
    }

    pushError(errmsg) {
        console.log(errmsg);
        this.errors.push(errmsg);
    }

    /** Ensures that immediate is 16 bits */
    normalizeImm(imm) {
        if (imm > 0xffff) {
            this.pushError("Immediate is more than 16 bits [line " + this.line + "]: " + imm);
        }
        return imm & 0xffff;
    }

    /** Verifies that an offset is valid (multiple of 4) */
    verifyOffset(offset) {
        if (offset % 4 !== 0) {
            this.pushError("Misaligned branch offset (must be a multiple of 4) [line " + this.line + "]: " + offset);
            return false;
        }
        return true;
    }

    /** Verifies that a pc is valid */
    verifyPC(pc) {
        return (0 <= pc / 4 && pc % 4 == 0);
    }

    /** Verifies that there is another delay slot in progress
     *  No need for delay slot though.
    verifyDelaySlot() {
        if (this.delaySlot) {
            this.pushError("Cannot have a jump/branch instruction in delay slot! [line " + this.line + "]. Ignoring jump/branch in delay slot.");
            return true;
        }
        return false;
    }
    */

    /** Verifies a memory range from loc1 -> loc2 */
    verifyMemory(loc1, loc2) {
        if (!this.memory.isValidAddress(loc1) || !this.memory.isValidAddress(loc2)) {
            this.pushError("Invalid memory location [line " + this.line + "]: " + (loc1 >>> 0) +
                    ((loc2 === undefined) ? "" : " to " + (loc2 >>> 0)));
        }
    }

    // j(target) {
    //     if (!this.verifyDelaySlot()) { // only execute jump if this is not a delay slot instruction
    //         this.delaySlot = true;
    //         var newpc = (this.pc & 0xf0000000) + target; // pc already points to instruction in delay slot
    //         if (!this.verifyPC(newpc)) {
    //             this.pushError("Misaligned jump target (must be a multiple of 4 and in program range) [line " + this.line + "]: " + target);
    //         }
    //         else {
    //             this.delaySlotInsnsPC.push(this.pc); // note that a delay slot was executed for the client view
    //             this.step();
    //             this.pc = newpc;
    //         }
    //         this.delaySlot = false;
    //     }
    // }

    // jr(rs) {
    //     if (!this.verifyDelaySlot()) { // only execute jump if this is not a delay slot instruction
    //         this.delaySlot = true;
    //         var newpc = this.registers[rs] >>> 0;
    //         if (!this.verifyPC(newpc)) {
    //             this.pushError("Bad PC value to jump to for register " + rs + " (must be a multiple of 4 and in program range) [line " + this.line + "]: " + newpc);
    //         }
    //         else {
    //             this.delaySlotInsnsPC.push(this.pc); // note that a delay slot was executed for the client view
    //             this.step();
    //             this.pc = newpc;
    //         }
    //         this.delaySlot = false;
    //     }
    // }


    // //TODO: Remove the delay slot consideration
    // jal(target) {
    //     if (!this.verifyDelaySlot()) { // only change xra if this is not a delay slot instruction
    //         this.registers[31] = this.pc + 4; // pc was already incremented by 4, so xra is pc + 8 (second instruction after jump)
    //         this.j(target);
    //     }
    // }

    // //TODO: Remove the delay slot consideration
    // jalr(rd, rs) {
    //     if (!this.verifyDelaySlot()) { // only change xra if this is not a delay slot instruction
    //         if (rd === rs) {
    //             this.pushError("jalr instruction cannot have the same values for rs and rd [line " + this.line + "]");
    //         }
    //         this.registers[rd] = this.pc + 4; // pc was already incremented by 4, so xra is pc + 8 (second instruction after jump)
    //         this.jr(rs);
    //     }
    // }

    parseRegister(tok) {
        switch(tok) {
            case "zero":
            case "x0":
                return 0;
            case "ra":
            case "x1":
                return 1;
            case "sp":
            case "x2":
                return 2;
            case "gp":
            case "x3":
                return 3;
            case "tp":
            case "x4":
                return 4;
            case "t0":
            case "x5":
                return 5;
            case "t1":
            case "x6":
                return 6;
            case "t2":
            case "x7":
                return 7;
            case "s0":
            case "fp":
            case "x8":
                return 8;
            case "s1":
            case "x9":
                return 9;
            case "a0":
            case "x10":
                return 10;
            case "a1":
            case "x11":
                return 11;
            case "a2":
            case "x12":
                return 12;
            case "a3":
            case "x13":
                return 13;
            case "a4":
            case "x14":
                return 14;
            case "a5":
            case "x15":
                return 15;
            case "a6":
            case "x16":
                return 16;
            case "a7":
            case "x17":
                return 17;
            case "s2":
            case "x18":
                return 18;
            case "s3":
            case "x19":
                return 19;
            case "s4":
            case "x20":
                return 20;
            case "s5":
            case "x21":
                return 21;
            case "s6":
            case "x22":
                return 22;
            case "s7":
            case "x23":
                return 23;
            case "s8":
            case "x24":
                return 24;
            case "s9":
            case "x25":
                return 25;
            case "s10":
            case "x26":
                return 26;
            case "s11":
            case "x27":
                return 27;
            case "t3":
            case "x28":
                return 28;
            case "t4":
            case "x29":
                return 29;
            case "t5":
            case "x30":
                return 30;
            case "t6":
            case "x31":
                return 31;
        }
        this.pushError("Invalid register [line " + this.line + "]: " + tok);
        return undefined; // invalid register
    }

    //Returns either if its an immediate or register, and its corresponding value
    parseToken(tok) {
        var value;
        var type;
        if (tok.charAt(0) == 'x') { //Toekn before
            value = this.parseRegister(tok);
            type = this.TOKEN_TYPE_REG;
        }
        else {
            if("abcdefghijklmnopqrstuvwxyz".indexOf(tok.charAt(0).toLowerCase()) > -1){
                value = this.parseRegister(tok);

                type = this.TOKEN_TYPE_REG;

                if (value === undefined) {
                    this.pushError("Unknown value [line " + this.line + "]: " + tok);
                }
            }else{
                value = parseInt(tok);

                type = this.TOKEN_TYPE_IMM;
                if (value === undefined) {
                    this.pushError("Unknown value [line " + this.line + "]: " + tok);
                }
            }
        }
        return {value: value, type: type};
    }

    verifyTokenTypes(tokens, types, format) {
        if (tokens.length < types.length) {
            this.pushError("Too few arguments [line " + this.line + "] for '" + format + "'");
        }
        if (tokens.length > types.length) {
            this.pushError("Extra arguments [line " + this.line + "] for '" + format + "': " + tokens.slice(types.length, tokens.length).join(', '));
        }
        for (var i = 0; i < tokens.length; ++i) {
            if (tokens[i].type !== types[i]) {
                this.pushError("Incorrect argument type [line " + this.line + "] for '" + format + "'");
            }
            tokens[i] = tokens[i].value;
        }
    }
    step() {
      this.numCycles=this.numCycles+1;
      console.log(this.numCycles);
        if (!this.verifyPC(this.pc) || this.pc / 4 >= this.insns.length ) {
            console.log("PC is invalid!! PC = " + this.pc);
            return;
        }
        var insn = this.insns[this.pc / 4][0];
        this.line = this.insns[this.pc / 4][1];
        this.pc += 4;
        if (insn.indexOf(' ') != -1) { // if not bad format, since all instructions have a space after the op
            var op = insn.substring(0, insn.indexOf(' '));
            var stringTokens = insn.substring(insn.indexOf(' '), insn.length).split(",");
            var tokens = [];
            var tokensIndex = 0;
            for (var i = 0; i < stringTokens.length; ++i) {
                var trimmed = stringTokens[i].trim();
                if (trimmed.indexOf('#') != -1) { // remove end of line comments
                    trimmed = trimmed.substring(0, trimmed.indexOf('#')).trim();
                    tokens[tokensIndex] = this.parseToken(trimmed);
                    break;
                }
                else if (trimmed.indexOf('(') != -1 && trimmed.indexOf(')') != -1) { // location of memory for load/store operations: offset($register)
                    tokens[tokensIndex] = this.parseToken(trimmed.substring(0, trimmed.indexOf('('))); // parse the offset
                    tokensIndex++;
                    tokens[tokensIndex] = this.parseToken(trimmed.substring(trimmed.indexOf('(')+1, trimmed.indexOf(')'))); // parse the register
                }
                else { // parses a single register or immediate value
                    tokens[tokensIndex] = this.parseToken(trimmed);
                }
                tokensIndex++;
            }
            switch(op.trim().toLowerCase()) {
                // case "jalr":
                //     // if (tokens.length == 1) {
                //     //     tokens.unshift({value: 31, type: this.TOKEN_TYPE_REG}); // use $31 as $rd
                //     // }
                //     this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG], "jalr $rd, $rs");
                //     this.jalr(tokens[0], tokens[1]);
                //     break;
                // case "jal":
                //     this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_IMM], "jal target");
                //     this.jal(tokens[0]);
                //     break;
                case "add":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG], "add rd, rs1, rs2");
                    this.add(tokens[0], tokens[1], tokens[2]);
                    break;
                case "addi":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "addi rd, rs1, immediate");
                    this.addi(tokens[0], tokens[1], tokens[2]);
                    break;
                case "sub":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG], "add rd, rs1, rs2");
                    this.sub(tokens[0], tokens[1], tokens[2]);
                    break;
                case "mult":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG], "add rd, rs1, rs2");
                    this.mult(tokens[0], tokens[1], tokens[2]);
                    break;
                case "and":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG], "add rd, rs1, rs2");
                    this.and(tokens[0], tokens[1], tokens[2]);
                    break;
                case "or":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG], "add rd, rs1, rs2");
                    this.or(tokens[0], tokens[1], tokens[2]);
                    break;
                case "xor":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG], "add rd, rs1, rs2");
                    this.xor(tokens[0], tokens[1], tokens[2]);
                    break;
                case "andi":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "add rd, rs1, imm");
                    this.andi(tokens[0], tokens[1], tokens[2]);
                    break;
                case "ori":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "add rd, rs1, imm");
                    this.ori(tokens[0], tokens[1], tokens[2]);
                    break;
                case "xori":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "add rd, rs1, imm");
                    this.xori(tokens[0], tokens[1], tokens[2]);
                    break;
                case "slt":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG], "slt rd, rs1, rs2");
                    this.slt(tokens[0], tokens[1], tokens[2]);
                    break;
                case "slti":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "slti rd, rs1, imm");
                    this.slti(tokens[0], tokens[1], tokens[2]);
                    break;
                case "sltu":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG], "sltu rd, rs1, rs2");
                    this.sltu(tokens[0], tokens[1], tokens[2]);
                    break;
                case "sltiu":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "sltiu rd, rs1, imm");
                    this.sltiu(tokens[0], tokens[1], tokens[2]);
                    break;
                case "sll":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG], "sll rd, rs1, rs2");
                    this.sll(tokens[0], tokens[1], tokens[2]);
                    break;
                case "slli":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "slli rd, rs1, imm");
                    this.slli(tokens[0], tokens[1], tokens[2]);
                    break;
                case "srl":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG], "srl rd, rs1, rs2");
                    this.srl(tokens[0], tokens[1], tokens[2]);
                    break;
                case "srli":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "srli rd, rs1, imm");
                    this.srli(tokens[0], tokens[1], tokens[2]);
                    break;
                case "sra":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG], "sra rd, rs1, rs2");
                    this.sra(tokens[0], tokens[1], tokens[2]);
                    break;
                case "srai":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "srai rd, rs1, imm");
                    this.srai(tokens[0], tokens[1], tokens[2]);
                    break;
                case "lw":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM, this.TOKEN_TYPE_REG], "lw rs1, rs2, imm");
                    this.lw(tokens[0], tokens[1], tokens[2]);
                    break;
                case "sw":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM, this.TOKEN_TYPE_REG], "sw rs1, rs2, imm");
                    this.sw(tokens[0], tokens[1], tokens[2]);
                    break;
                case "lb":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM, this.TOKEN_TYPE_REG], "lw rs1, rs2, imm");
                    this.lb(tokens[0], tokens[1], tokens[2]);
                    break;
                case "sb":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM, this.TOKEN_TYPE_REG], "sw rs1, rs2, imm");
                    this.sb(tokens[0], tokens[1], tokens[2]);
                    break;
                case "beq":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "beq rs1, rs2, imm");
                    this.beq(tokens[0], tokens[1], tokens[2]);
                    break;
                case "bne":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "bne rs1, rs2, imm");
                    this.bne(tokens[0], tokens[1], tokens[2]);
                    break;
                case "blt":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "blt rs1, rs2, imm");
                    this.blt(tokens[0], tokens[1], tokens[2]);
                    break;
                case "bge":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "bge rs1, rs2, imm");
                    this.bge(tokens[0], tokens[1], tokens[2]);
                    break;
                case "bltu":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "blt rs1, rs2, imm");
                    this.bltu(tokens[0], tokens[1], tokens[2]);
                    break;
                case "bgeu":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "bge rs1, rs2, imm");
                    this.bgeu(tokens[0], tokens[1], tokens[2]);
                    break;
                case "jal":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "JAL rd, imm");
                    this.jal(tokens[0], tokens[1]);
                    break;
                case "jalr":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "JALR rd, rs1, imm");
                    this.jalr(tokens[0], tokens[1], tokens[2]);
                    break;
                case "lui":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "lui rd, imm");
                    this.lui(tokens[0], tokens[1]);
                    break;
                case "auipc":
                    this.verifyTokenTypes(tokens, [this.TOKEN_TYPE_REG, this.TOKEN_TYPE_IMM], "auipc rd, rs1");
                    this.auipc(tokens[0], tokens[1]);
                    break;
                default:
                    this.pushError("Unsupported Op [line " + this.line +"]: " + op);
            }
        }
    }

    createBinaryString(nMask) {
        // nMask must be between -2147483648 and 2147483647
        for (var nFlag = 0, nShifted = nMask, sMask = ""; nFlag < 32;
             nFlag++, sMask += String(nShifted >>> 31), nShifted <<= 1);
        return sMask;
    }

    /* Adds an error message if the immediate is too large for the instruction */
    error_check_limit_imm(imm, num_bits, op){
        // TODO: Get this working with both positive and negative numbers
        // if(imm < 0){
        //     const check1 = this.createBinaryString(this.bit_limit_imm(imm, num_bits)).slice(32 - num_bits, 32);
        //     const check2 = this.createBinaryString(imm).slice(32 - num_bits, 32);
        //     if(check1 != check2){
        //         this.pushError("Immediate max length for operation '" + op + "' is " + num_bits + " bits long - [line " + this.line +"]");
        //     }
        // }else{
        //     if(this.bit_limit_imm(imm, num_bits) != imm){
        //         this.pushError("Immediate max length for operation '" + op + "' is " + num_bits + " bits long - [line " + this.line +"]");
        //     }
        // }
    }

    /* Truncates an immediate number of any length to num bits */
    bit_limit_imm(imm, num_bits){
        imm = imm >>> 0;
        var mask = (1 << num_bits) - 1;
        return imm & mask
    }

    /* Extends an immediate number of num bits to 32 bits */
    bit_ext_imm(imm, num_bits){
        var msb = 1 << (num_bits - 1); //gives binary n digits, with msb as 1 and (n-1) 0s
        if((imm & msb) == msb){
            var ext = (1 << (32 - num_bits)) - 1 << num_bits; //gets the top bits for sign
            imm = imm | ext;
        }
        return imm;
    }

    /* imm is sign extended */
    addi(rd, rs1, imm) {
        //console.log(imm)
        //this.error_check_limit_imm(imm, 12, "addi")
        imm = this.bit_limit_imm(imm, 12);
        imm = this.bit_ext_imm(imm, 12);
        //console.log(imm)
        //console.log(this.registers[rs1] + imm)
        if(rd != 0) this.registers[rd] = this.registers[rs1] + imm;
    }

    add(rd, rs1, rs2){
        if(rd != 0) this.registers[rd] = this.registers[rs1] + this.registers[rs2];
    }

    sub(rd, rs1, rs2){
        if(rd != 0) this.registers[rd] = this.registers[rs1] - this.registers[rs2];
    }

    mult(rd, rs1, rs2){
        if(rd != 0) this.registers[rd] = this.registers[rs1] * this.registers[rs2];
    }

    and(rd, rs1, rs2) {
        if(rd != 0) this.registers[rd] = this.registers[rs1] & this.registers[rs2];
    }

    or(rd, rs1, rs2) {
        if(rd != 0) this.registers[rd] = this.registers[rs1] | this.registers[rs2];
    }

    xor(rd, rs1, rs2) {
        if(rd != 0) this.registers[rd] = this.registers[rs1] ^ this.registers[rs2];
    }

    andi(rd, rs1, imm) {
        //this.error_check_limit_imm(imm, 12, "andi")
        imm = this.bit_limit_imm(imm, 12);
        imm = this.bit_ext_imm(imm, 12);
        if(rd != 0) this.registers[rd] = this.registers[rs1] & imm;
    }

    /* ORI
    - Summary   : Bitwise logical OR with constant
    - Assembly  : ori rd, rs1, imm
    - Semantics : R[rd] = R[rs1] | sext(imm)
    - Format    : I-type, I-immediate
    */
    ori(rd, rs1, imm) {
        //this.error_check_limit_imm(imm, 12, "ori")
        imm = this.bit_limit_imm(imm, 12);
        imm = this.bit_ext_imm(imm, 12);
        if(rd != 0) this.registers[rd] = this.registers[rs1] | imm;
    }

    xori(rd, rs1, imm) {
        //this.error_check_limit_imm(imm, 12, "xori")
        imm = this.bit_limit_imm(imm, 12);
        imm = this.bit_ext_imm(imm, 12);
        if(rd != 0) this.registers[rd] = this.registers[rs1] ^ imm;
    }

    slt(rd, rs1, rs2){
        if(rd != 0) this.registers[rd] = (this.registers[rs1] < this.registers[rs2]) ? 1 : 0;
    }

    slti(rd, rs1, imm){
        //this.error_check_limit_imm(imm, 12, "slti")
        imm = this.bit_limit_imm(imm, 12);
        imm = this.bit_ext_imm(imm, 12);
        if(rd != 0) this.registers[rd] = (this.registers[rs1] < imm) ? 1 : 0;
    }

    sltiu(rd, rs1, imm){
        //this.error_check_limit_imm(imm, 12, "sltiu")
        imm = this.bit_limit_imm(imm, 12);
        imm = this.bit_ext_imm(imm, 12);
        if(rd != 0) this.registers[rd] = ((this.registers[rs1] >>> 0) < (imm >>> 0) ) ? 1 : 0;
    }

    sltu(rd, rs1, rs2){
        if(rd != 0) this.registers[rd] = ( (this.registers[rs1] >>> 0) < (this.registers[rs2] >>> 0) ) ? 1 : 0;
    }

    sll(rd, rs1, rs2){
        if(rd != 0) this.registers[rd] = this.registers[rs1] << (this.registers[rs2] & 0x0000001f);
    }

    slli(rd, rs1, imm){
        //this.error_check_limit_imm(imm, 5, "slli")
        imm = this.bit_limit_imm(imm, 5);
        imm = this.bit_ext_imm(imm, 5);
        if(rd != 0) this.registers[rd] = this.registers[rs1] << imm;
    }

    srl(rd, rs1, rs2){
        if(rd != 0) this.registers[rd] = this.registers[rs1] >>> (this.registers[rs2] & 0x0000001f);
    }

    srli(rd, rs1, imm){
        //this.error_check_limit_imm(imm, 5, "srli")
        imm = this.bit_limit_imm(imm, 5);
        imm = this.bit_ext_imm(imm, 5);
        if(rd != 0) this.registers[rd] = this.registers[rs1] >>> imm;
    }

    sra(rd, rs1, rs2){
        if(rd != 0) this.registers[rd] = this.registers[rs1] >> (this.registers[rs2] & 0x0000001f);
    }

    srai(rd, rs1, imm){
        //this.error_check_limit_imm(imm, 5, "srai")
        imm = this.bit_limit_imm(imm, 5);
        imm = this.bit_ext_imm(imm, 5);
        if(rd != 0) this.registers[rd] = this.registers[rs1] >> imm;
    }

    auipc(rd, imm){
        imm = this.bit_limit_imm(imm, 20);
        imm = this.bit_ext_imm(imm, 20);
        if (rd != 0) this.registers[rd] = (this.pc - 4) + (imm << 12);
    }

    /*
    For best performance, the effective address for all loads and stores should be
    naturally aligned for each data type (i.e., on a four-byte boundary for 32-bit
        accesses, and a two-byte boundary for 16-bit accesses). The base ISA supports
         misaligned accesses, but these might run extremely slowly depending on the
         implementation.
    */

    /*
    * LW
    - Summary   : Load word from memory
    - Assembly  : lw rd, imm(rs1)
    - Semantics : R[rd] = M_4B[ R[rs1] + sext(imm) ]
    - Format    : I-type, I-immediate
    */
    lw(rd, imm, rs1) {
        //this.error_check_limit_imm(imm, 12, "lw")
        imm = this.bit_limit_imm(imm, 12);
        imm = this.bit_ext_imm(imm, 12);

        var loc = imm + this.registers[rs1];
        if (loc % 4 != 0 ) {
            this.pushError("Address Error: loading from non-aligned word [line " + this.line + "]: " + loc);
        }
        this.verifyMemory(loc, loc+3);
        var lsb = this.memory.getMem(loc);
        var byte2 = this.memory.getMem(loc+1) << 8;
        var byte3 = this.memory.getMem(loc+2) << 16;
        var msb = this.memory.getMem(loc+3) << 24;
        if(rd != 0) this.registers[rd] = msb + byte3 + byte2 + lsb;
    }

    /*
    * SW
    - Summary   : Store word into memory
    - Assembly  : sw rs2, imm(rs1)
    - Semantics : M_4B[ R[rs1] + sext(imm) ] = R[rs2]
    - Format    : S-type, S-immediate
    */
    sw(rs2, imm, rs1) {
        //this.error_check_limit_imm(imm, 12, "sw")
        imm = this.bit_limit_imm(imm, 12);
        imm = this.bit_ext_imm(imm, 12);

        var registerValue = this.registers[rs2];
        var loc = imm + this.registers[rs1];
        if (loc % 4 != 0 ) {
            this.pushError("Address Error: storing to non-aligned word [line " + this.line + "]: " + loc);
        }
        this.verifyMemory(loc, loc+3);
        this.memory.setMem(loc, registerValue & 0x000000ff);
        this.memory.setMem(loc+1, (registerValue >>> 8) & 0x000000ff);
        this.memory.setMem(loc+2, (registerValue >>> 16) & 0x000000ff);
        this.memory.setMem(loc+3, (registerValue >>> 24) & 0x000000ff);
    }

    lb(rd, imm, rs1) {
        imm = this.bit_limit_imm(imm, 12);
        imm = this.bit_ext_imm(imm, 12);
        var loc = imm + this.registers[rs1];
        this.verifyMemory(loc);
        if(rd != 0) this.registers[rd] = this.bit_ext_imm(this.memory.getMem(loc), 8);
    }

    sb(rs2, imm, rs1) {
        imm = this.bit_limit_imm(imm, 12);
        imm = this.bit_ext_imm(imm, 12);
        var loc = imm + this.registers[rs1];
        this.verifyMemory(loc);
        this.memory.setMem(loc, this.registers[rs2] & 0x000000ff);
    }

    branch(rs1, rs2, offset, predicate){
        if (this.verifyOffset(offset)) {
            //offset = this.offsetPad(offset);
            var newpc = (this.pc - 4) + offset; //offset used to be relative to delay slot - what now?
            if (!this.verifyPC(newpc)) {
                this.pushError("Bad branch offset (must be in program range) [line " + this.line + "]: " + offset);
            }
            else {
                var branch = false;
                if (predicate(rs1, rs2)) {
                    branch = true;
                }
                if (branch) {
                    this.pc = newpc;
                }
            }
            this.delaySlot = false;
        }
    }

    beq(rs1, rs2, offset) {
        //this.error_check_limit_imm(offset, 12, "beq")
        offset = this.bit_limit_imm(offset, 13);
        offset = this.bit_ext_imm(offset, 13);
        this.branch(rs1, rs2, offset, (x, y) => this.registers[x] == this.registers[y]);
    }

    bne(rs1, rs2, offset) {
        //this.error_check_limit_imm(offset, 12, "bne")
        offset = this.bit_limit_imm(offset, 13);
        offset = this.bit_ext_imm(offset, 13);
        this.branch(rs1, rs2, offset, (x, y) => this.registers[x] != this.registers[y]);
    }

    blt(rs1, rs2, offset) {
        //this.error_check_limit_imm(offset, 12, "blt")
        offset = this.bit_limit_imm(offset, 13);
        offset = this.bit_ext_imm(offset, 13);
        this.branch(rs1, rs2, offset, (x, y) => this.registers[x] < this.registers[y]);
    }

    bge(rs1, rs2, offset) {
        //this.error_check_limit_imm(offset, 10, "bge")
        offset = this.bit_limit_imm(offset, 13);
        offset = this.bit_ext_imm(offset, 13);
        this.branch(rs1, rs2, offset, (x, y) => this.registers[x] >= this.registers[y]);
    }

    bltu(rs1, rs2, offset) {
        // offset = this.bit_limit_imm(offset, 10);
        // offset = this.bit_ext_imm(offset, 10);
        if(this.registers[rs1] == this.registers[rs2]){
            this.branch(rs1, rs2, offset, (x, y) => {return false;});
        }else{
            this.bgeu(rs2, rs1, offset);
        }
    }

    bgeu(rs1, rs2, offset) {
        //this.error_check_limit_imm(offset, 10, "bge")
        offset = this.bit_limit_imm(offset, 13);
        offset = this.bit_ext_imm(offset, 13);
        this.branch(rs1, rs2, offset, (x, y) => {
            var temp_x = this.registers[x];
            var temp_y = this.registers[y];
            if(temp_x < 0){
                if(temp_y >= 0){
                    return true
                }else if(temp_y == 0){
                    return true;
                }else{
                    return temp_x >= temp_y;
                }
            }else if(temp_x > 0){
                if(temp_y < 0){
                    return false;
                }else if(temp_y == 0){
                    return true;
                }else{
                    return temp_x >= temp_y;
                }
            }else{
                return temp_x == temp_y; //if x == 0, then unsigned, could only be equal
            }
        });
    }

    /*
    * JAL
    - Summary   : Jump to address and place return address in GPR
    - Assembly  : jal rd, imm
    - Semantics : R[rd] = PC + 4; PC = PC + sext(imm)
    - Format    : U-type, J-immediate

    Assumption that pc has already been incremented + 4
    If imm is not 4-byte aligned, pc effectly becomes (pc - (pc % 4))

    curr pc = pc - 4

    */
    jal(rd, imm) {

        let temp_pc = this.pc;

        //this.error_check_limit_imm(imm, 20, "jal")
        imm = this.bit_limit_imm(imm, 21);
        imm = this.bit_ext_imm(imm, 21);
        // console.log(imm)
        // console.log(this.pc)
        // console.log(this.pc - 4)
        var newpc = (this.pc - 4) + imm; //negate delay slot
        if (!this.verifyPC(newpc)) {
            this.pushError("Misaligned jump target (must be a multiple of 4 and in program range) [line " + this.line + "]: " + imm);
        }
        else {
            this.pc = newpc;
        }

        if(rd != 0){
            this.registers[rd] = temp_pc; //very next instruciton
        }
    }

    /*
    * JALR
    - Summary   : Jump to address and place return address in GPR
    - Assembly  : jalr rd, rs1, imm
    - Semantics : R[rd] = PC + 4; PC = ( R[rs1] + sext(imm) ) & 0xfffffffe
    - Format    : I-Type, I-immediate

    Assumption that pc has already been incremented + 4
    If imm is not 4-byte aligned, pc effectly becomes (pc - (pc % 4))
    */
    jalr(rd, rs1, imm) {

        let temp_pc = this.pc;

        //this.error_check_limit_imm(imm, 20, "jal")
        imm = this.bit_limit_imm(imm, 21);
        imm = this.bit_ext_imm(imm, 21);

        var newpc = (this.registers[rs1] + imm) & 0xfffffffe;

        if (!this.verifyPC(newpc)) {
            this.pushError("Bad PC value to jump to for register " + rs + " (must be a multiple of 4 and in program range) [line " + this.line + "]: " + newpc);
        }
        else {
            this.pc = newpc;
        }

        if(rd != 0){
            this.registers[rd] = temp_pc; //already added 4 by this point
        }
    }

    /*
    * LUI
    - Summary   : Load constant into upper bits of word
    - Assembly  : lui rd, imm
    - Semantics : R[rd] = imm << 12
    - Format    : I-type, U-immediate
    */
    lui(rd, imm){
        //this.error_check_limit_imm(imm, 20, "lui")
        imm = this.bit_limit_imm(imm, 20);
        if(rd != 0){
            this.registers[rd] = imm << 12;
        }
    }

    /** Not used in the main browser runner */
    run() {
        while ((this.pc / 4) < this.insns.length) {
            this.step();
        }
    }
}
