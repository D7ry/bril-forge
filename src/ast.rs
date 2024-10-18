#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::hash_set::HashSet;
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
    pub label: Option<String>, // label which other bb's use to jump in to this bb
    pub out_labels: Vec<String>, // label which this bb is able to jump to
    pub in_bb_indices: HashSet<usize>, // indices into the function's bb that jumps to this bb
    pub out_bb_indices: HashSet<usize>, // indices into the function's bb that this bb jumps out to
    pub immediate_dominator_index: usize, // index into the function's bb that immediately
                                          // dominates this bb.
}

impl BasicBlock {
    pub fn new() -> BasicBlock {
        BasicBlock {
            instrs: Vec::new(),
            label: None,
            out_labels: Vec::new(),
            in_bb_indices: HashSet::new(),
            out_bb_indices: HashSet::new(),
            immediate_dominator_index: 0
        }
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

        // <basic block's in label, indices to `ret` of the corresponding basic block
        let mut bb_labels_to_indices: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for inst in self.instrs.iter() {
            match (inst.is_label(), inst.is_control_inst()) {
                (true, true) => {
                    panic!("instruction cannot be both a label and a control instruction!");
                }
                (true, _) => {
                    // only start a new block if the label would
                    // otherwise break the current BB's invariant
                    if !current_block.instrs.is_empty() {
                        match inst {
                            Instruction::Label { label } => {
                                // the block jumps to this label and gg
                                current_block.out_labels.push(label.clone());
                            }
                            _ => {
                                panic!("instruction has to be label to reach here");
                            }
                        }
                        if let Some(block_label) = current_block.label.clone() {
                            bb_labels_to_indices.insert(block_label, ret.len());
                        }
                        ret.push(current_block);
                        current_block = BasicBlock::new();
                        //NOTE: this is to handle special case where an anonymous
                        //bb jumps to another bb. we track its index 
                        current_block.in_bb_indices.insert(ret.len() - 1); // the previous bb is
                                                                           // the bb's in bb
                    }
                    // push label inst
                    current_block.instrs.push(inst.clone());
                    current_block.label = inst.get_result(); // mark in label
                }
                (_, true) => {
                    // is control
                    // push control inst to current block
                    current_block.instrs.push(inst.clone());
                    match inst {
                        Instruction::Opcode(inst) => match inst {
                            OpcodeInstruction::Jmp { labels } => {
                                current_block.out_labels = labels.clone();
                            }
                            OpcodeInstruction::Br { labels, .. } => {
                                current_block.out_labels = labels.clone();
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                    // shouldn't happen but just in case
                    if let Some(label) = current_block.label.clone() {
                        bb_labels_to_indices.insert(label, ret.len());
                    }
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

        // Step 1: Collect indices
        let mut parent_to_child_indices: Vec<(usize, usize)> = Vec::new();

        for (bb_index, bb) in ret.iter_mut().enumerate() {
            for label in &bb.out_labels {
                if let Some(successor_index) = bb_labels_to_indices.get(label) {
                    parent_to_child_indices.push((bb_index, *successor_index));
                }
            }
            for idx in bb.in_bb_indices.iter() {
                parent_to_child_indices.push((idx.clone(), bb_index));
            }
        }

        // Step 2: Mutate using collected indices
        for (parent_index, child_index) in parent_to_child_indices {
            if let Some(bb) = ret.get_mut(parent_index) {
                bb.out_bb_indices.insert(child_index);
            }
            if let Some(bb) = ret.get_mut(child_index) {
                bb.in_bb_indices.insert(parent_index);
            }
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
impl ToString for Type {
    fn to_string(&self) -> String {
        match self {
            Type::Primitive(s) => s.clone(),
            Type::Pointer { ptr } => format!("ptr<{}>", ptr),
        }
    }
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
        value: Value, // can be `int`, `bool`, etc...
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

    pub fn is_meaningful(&self) -> bool {
        !self.has_no_side_effects()
    }

    // whether a instruction has "sideeffects"
    pub fn has_no_side_effects(&self) -> bool {
        match self {
            Instruction::Label { .. } => false,
            Instruction::Opcode(Inst) => match Inst {
                OpcodeInstruction::Print { .. }
                | OpcodeInstruction::Call { .. }
                | OpcodeInstruction::Ret { .. }
                | OpcodeInstruction::Store { .. }
                | OpcodeInstruction::Alloc { .. }
                | OpcodeInstruction::Free { .. } => false,
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
            Instruction::Opcode(Inst) => Inst.get_dest(),
            Instruction::Label { label } => Some(label.clone()),
            Instruction::Nop { .. } => Option::None,
        }
    }
}

impl OpcodeInstruction {
    // whether the instruction is asssigning some value on rhs expr
    // to lhs
    pub fn is_assignment_inst(&self) -> bool {
        self.get_dest().is_some()
    }
    // TODO: this has to go away need better granularity for OpcodeInstruction
    pub fn get_type(&self) -> Option<Type> {
        match self {
            OpcodeInstruction::Const { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Alloc { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Call { typ, .. } => typ.clone(),
            OpcodeInstruction::Print { .. } => None,
            OpcodeInstruction::Free { .. } => None,
            OpcodeInstruction::Ret { .. } => None,
            OpcodeInstruction::Id { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Store { .. } => None,
            OpcodeInstruction::Ptradd { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Br { .. } => None,
            OpcodeInstruction::Or { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Add { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Sub { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Div { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Mul { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::FAdd { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::FSub { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::FDiv { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::FMul { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Eq { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Gt { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Ge { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Lt { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Le { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::FEq { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::FGt { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::FGe { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::FLt { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::FLe { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::And { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Not { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Load { typ, .. } => Some(typ.clone()),
            OpcodeInstruction::Jmp { .. } => None,
        }
    }
    pub fn get_dest(&self) -> Option<String> {
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
            OpcodeInstruction::Br { args, labels } => {
                let mut uses: Vec<String> = args.to_vec(); // branch also use label insts
                labels.iter().for_each(|label| uses.push(label.clone()));
                uses
            }
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
            OpcodeInstruction::Jmp { labels } => labels.to_vec(),
        }
    }

    // returns mutable reference to function arguments
    pub fn get_args(&mut self) -> Option<&mut Vec<String>> {
        match self {
            OpcodeInstruction::Alloc { args, .. }
            | OpcodeInstruction::Call {
                args: Some(args), ..
            }
            | OpcodeInstruction::Print { args }
            | OpcodeInstruction::Free { args }
            | OpcodeInstruction::Ret { args }
            | OpcodeInstruction::Id { args, .. }
            | OpcodeInstruction::Store { args }
            | OpcodeInstruction::Ptradd { args, .. }
            | OpcodeInstruction::Br { args, .. }
            | OpcodeInstruction::Or { args, .. }
            | OpcodeInstruction::Add { args, .. }
            | OpcodeInstruction::Sub { args, .. }
            | OpcodeInstruction::Div { args, .. }
            | OpcodeInstruction::Mul { args, .. }
            | OpcodeInstruction::FAdd { args, .. }
            | OpcodeInstruction::FSub { args, .. }
            | OpcodeInstruction::FDiv { args, .. }
            | OpcodeInstruction::FMul { args, .. }
            | OpcodeInstruction::Eq { args, .. }
            | OpcodeInstruction::Gt { args, .. }
            | OpcodeInstruction::Ge { args, .. }
            | OpcodeInstruction::Lt { args, .. }
            | OpcodeInstruction::Le { args, .. }
            | OpcodeInstruction::FEq { args, .. }
            | OpcodeInstruction::FGt { args, .. }
            | OpcodeInstruction::FGe { args, .. }
            | OpcodeInstruction::FLt { args, .. }
            | OpcodeInstruction::FLe { args, .. }
            | OpcodeInstruction::And { args, .. }
            | OpcodeInstruction::Not { args, .. }
            | OpcodeInstruction::Load { args, .. } => Some(args),

            // Handle the Call variant separately when `args` is None
            OpcodeInstruction::Call { args: None, .. } => None,

            // Variants without an `args` field
            _ => None,
        }
    }
}
