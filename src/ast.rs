#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, Read, Write};

// reads a program from a pipe, if not successful panik
pub fn read_from_pipe() -> Program {
    let mut buffer = String::new();
    let _ = io::stdin().read_to_string(&mut buffer).unwrap();
    let program: Program = serde_json::from_str(&buffer).unwrap();
    return program;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Program {
    pub functions: Vec<Function>,
}

impl Program {
    pub fn dump(&self) {
        for function in &self.functions {
            println!("Function: {}", function.name);
            for (i, instr) in function.instrs.iter().enumerate() {
                println!("  Instruction {}: {:?}", i, instr);
            }
        }
    }
    // write json to stdout or blow up the program
    pub fn dump_json(&self) {
        let json = serde_json::to_string(&self).unwrap();
        io::stdout().write_all(json.as_bytes()).unwrap();
        io::stdout().flush().unwrap();
    }
}

#[derive(Debug)]
pub struct BasicBlock {
    pub instrs: Vec<Instruction>,
}

impl BasicBlock {
    pub fn new() -> BasicBlock {
        BasicBlock { instrs: Vec::new() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<Argument>>,
    pub instrs: Vec<Instruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub return_type: Option<Type>,
}

impl Function {
    // get all the basic blocks of the function
    // note that instructions in BB are cloned instructions
    // so only may use it for anlysis passes
    pub fn get_basic_blocks(&self) -> Vec<BasicBlock> {
        let mut ret: Vec<BasicBlock> = Vec::new();
        let mut current_block = BasicBlock::new();

        for inst in self.instrs.iter() {
            match (inst.is_label(), inst.is_control_inst()) {
                (true, _) => {
                    // only start a new block if the label would
                    // otherwise break the current BB's invariant
                    if !current_block.instrs.is_empty() {
                        ret.push(current_block);
                        current_block = BasicBlock::new();
                    }
                    // push label inst
                    current_block.instrs.push(inst.clone());
                }
                (_, true) => {
                    // is control
                    // push control inst to current block
                    current_block.instrs.push(inst.clone());
                    ret.push(current_block);
                    // end current block
                    current_block = BasicBlock::new();
                }
                _ => {
                    // For other instructions, add to the current block
                    current_block.instrs.push(inst.clone());
                }
            }
        }
        if !current_block.instrs.is_empty() {
            ret.push(current_block);
        }

        ret
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Argument {
    name: String,
    #[serde(rename = "type")]
    arg_type: Type,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Type {
    Primitive(String),
    Pointer { ptr: String },
    // other wrapper types?
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Instruction {
    // instruction can either have opcode, or just be label or nop
    Opcode(OpcodeInstruction),
    Label { label: String },
    Nop { op: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "op")]
#[serde(rename_all = "lowercase")]
pub enum OpcodeInstruction {
    // half of it generated with chatgipidy because im lazy
    #[serde(rename = "const")]
    Const {
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
        value: Value,
    },
    Alloc {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Call {
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dest: Option<String>,
        funcs: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "type")]
        typ: Option<Type>,
    },
    Print {
        args: Vec<String>,
    },
    Free {
        args: Vec<String>,
    },
    Ret {
        args: Vec<String>,
    },
    Id {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Store {
        args: Vec<String>,
    },
    Ptradd {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Br {
        args: Vec<String>,
        labels: Vec<String>,
    },
    Or {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Add {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Sub {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Div {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Mul {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FAdd {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FSub {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FDiv {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FMul {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Eq {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Gt {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Ge {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Lt {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Le {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FEq {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FGt {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FGe {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FLt {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FLe {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    And {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Not {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Load {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Jmp {
        labels: Vec<String>,
    },
}

impl Instruction {
    // jmp, br
    pub fn is_control_inst(&self) -> bool {
        if let Instruction::Opcode(Inst) = self {
            match Inst {
                OpcodeInstruction::Jmp { .. } | OpcodeInstruction::Br { .. } => true,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn is_label(&self) -> bool {
        match self {
            Instruction::Label { .. } => true,
            _ => false,
        }
    }

    pub fn is_pure(&self) -> bool {
        match self {
            Instruction::Opcode(Inst) => match Inst {
                // call inst has unpredictable behavior, so mark as inpure for now
                OpcodeInstruction::Print { .. } | OpcodeInstruction::Call { .. } => false,
                _ => {
                    if self.is_control_inst() {
                        false
                    } else {
                        true
                    }
                }
            },
            _ => true,
        }
    }

    pub fn get_use_list(&self) -> Vec<String> {
        match self {
            Instruction::Opcode(Inst) => Inst.get_use_list(),
            Instruction::Label { .. } => Vec::new(),
            Instruction::Nop { .. } => Vec::new(),
        }
    }

    pub fn get_result(&self) -> Option<String> {
        match self {
            Instruction::Opcode(Inst) => Inst.get_result(),
            Instruction::Label { .. } => Option::None,
            Instruction::Nop { .. } => Option::None,
        }
    }
}

impl OpcodeInstruction {
    pub fn get_result(&self) -> Option<String> {
        match self {
            OpcodeInstruction::Const { dest, .. }
            | OpcodeInstruction::Alloc { dest, .. }
            | OpcodeInstruction::Id { dest, .. }
            | OpcodeInstruction::Ptradd { dest, .. }
            | OpcodeInstruction::Or { dest, .. }
            | OpcodeInstruction::Add { dest, .. }
            | OpcodeInstruction::Sub { dest, .. }
            | OpcodeInstruction::Div { dest, .. }
            | OpcodeInstruction::Mul { dest, .. }
            | OpcodeInstruction::FAdd { dest, .. }
            | OpcodeInstruction::FSub { dest, .. }
            | OpcodeInstruction::FDiv { dest, .. }
            | OpcodeInstruction::FMul { dest, .. }
            | OpcodeInstruction::Eq { dest, .. }
            | OpcodeInstruction::Gt { dest, .. }
            | OpcodeInstruction::Ge { dest, .. }
            | OpcodeInstruction::Lt { dest, .. }
            | OpcodeInstruction::Le { dest, .. }
            | OpcodeInstruction::FEq { dest, .. }
            | OpcodeInstruction::FGt { dest, .. }
            | OpcodeInstruction::FGe { dest, .. }
            | OpcodeInstruction::FLt { dest, .. }
            | OpcodeInstruction::FLe { dest, .. }
            | OpcodeInstruction::And { dest, .. }
            | OpcodeInstruction::Not { dest, .. }
            | OpcodeInstruction::Load { dest, .. } => Some(dest.clone()),

            OpcodeInstruction::Print { .. }
            | OpcodeInstruction::Free { .. }
            | OpcodeInstruction::Ret { .. }
            | OpcodeInstruction::Store { .. }
            | OpcodeInstruction::Br { .. }
            | OpcodeInstruction::Jmp { .. } => None,

            // callInst's dest is an optional
            OpcodeInstruction::Call { dest, .. } => dest.clone(),
        }
    }
    pub fn get_use_list(&self) -> Vec<String> {
        // this is why i like C more, you just use a union to get to the args field
        match self {
            OpcodeInstruction::Const { .. } => Vec::new(),
            OpcodeInstruction::Alloc { args, .. } => args.to_vec(),
            OpcodeInstruction::Call { args, .. } => match args {
                Some(args) => args.to_vec(),
                None => Vec::new(),
            },
            OpcodeInstruction::Print { args } => args.to_vec(),
            OpcodeInstruction::Free { args } => args.to_vec(),
            OpcodeInstruction::Ret { args } => args.to_vec(),
            OpcodeInstruction::Id { args, .. } => args.to_vec(),
            OpcodeInstruction::Store { args } => args.to_vec(),
            OpcodeInstruction::Ptradd { args, .. } => args.to_vec(),
            OpcodeInstruction::Br { args, .. } => args.to_vec(),
            OpcodeInstruction::Or { args, .. } => args.to_vec(),
            OpcodeInstruction::Add { args, .. } => args.to_vec(),
            OpcodeInstruction::Sub { args, .. } => args.to_vec(),
            OpcodeInstruction::Div { args, .. } => args.to_vec(),
            OpcodeInstruction::Mul { args, .. } => args.to_vec(),
            OpcodeInstruction::FAdd { args, .. } => args.to_vec(),
            OpcodeInstruction::FSub { args, .. } => args.to_vec(),
            OpcodeInstruction::FDiv { args, .. } => args.to_vec(),
            OpcodeInstruction::FMul { args, .. } => args.to_vec(),
            OpcodeInstruction::Eq { args, .. } => args.to_vec(),
            OpcodeInstruction::Gt { args, .. } => args.to_vec(),
            OpcodeInstruction::Ge { args, .. } => args.to_vec(),
            OpcodeInstruction::Lt { args, .. } => args.to_vec(),
            OpcodeInstruction::Le { args, .. } => args.to_vec(),
            OpcodeInstruction::FEq { args, .. } => args.to_vec(),
            OpcodeInstruction::FGt { args, .. } => args.to_vec(),
            OpcodeInstruction::FGe { args, .. } => args.to_vec(),
            OpcodeInstruction::FLt { args, .. } => args.to_vec(),
            OpcodeInstruction::FLe { args, .. } => args.to_vec(),
            OpcodeInstruction::And { args, .. } => args.to_vec(),
            OpcodeInstruction::Not { args, .. } => args.to_vec(),
            OpcodeInstruction::Load { args, .. } => args.to_vec(),
            OpcodeInstruction::Jmp { .. } => Vec::new(),
        }
    }
}
