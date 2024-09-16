// local value numbering
use crate::ast;
use ast::*;
use std::collections::{HashMap, HashSet};

fn hash_expr(args: &Vec<String>, typ: &Type, opcode: &str) -> String {
    let mut hash: String = String::new();
    // opcode
    hash.push_str(&opcode);
    // type
    match typ {
        Type::Primitive(type_str) => {
            hash.push_str(&type_str);
        }
        Type::Pointer { ptr } => {
            hash.push_str("__pointer"); // ik it's jank
            hash.push_str(&ptr);
        }
    }
    
    // arguments
    args.iter().for_each(|arg| hash.push_str(arg));
    hash
}

fn hash_commutitative_expr(args: &Vec<String>, typ: &Type, opcode: &str) -> String {
    let mut args_sorted = args.clone();
    args_sorted.sort();
    let mut hash = String::new();
    args_sorted.iter().for_each(|arg| hash.push_str(arg));
    hash_expr(&args_sorted, typ, opcode)
}

fn get_rhs_hash(inst: &Instruction) -> Option<String> {
    match inst {
        Instruction::Opcode(opcode_inst) => match opcode_inst {
            // commutative operations
            OpcodeInstruction::Add { args, dest, typ } => {
                Some(hash_commutitative_expr(args, typ, "add"))
            }
            OpcodeInstruction::FAdd { args, dest, typ } => {
                Some(hash_commutitative_expr(args, typ, "Fadd"))
            }
            OpcodeInstruction::Mul { args, dest, typ } => {
                Some(hash_commutitative_expr(args, typ, "Mul"))
            }
            OpcodeInstruction::FMul { args, dest, typ } => {
                Some(hash_commutitative_expr(args, typ, "FMul"))
            }
            _ => None,
        },
        _ => None,
    }
}

// perform lvn on bb that does the following optimizations:
//
//
// Note the pass doens't assume SSA
fn lvn_bb(bb: &mut BasicBlock) -> bool {
    let mut changed: bool = false;

    let expression_to_value_number: HashMap<String, u32> = HashMap::new();
    let variable_to_value_number: HashMap<String, u32> = HashMap::new();

    // build up the lvn table, good old spir-v time
    let i: u32 = 0; // counter for value number

    for inst in bb.instrs.iter_mut() {
        if let Some(rhs_hash) = get_rhs_hash(inst) {
            // loop up expr in value number
            
        }
    }
    changed
}

fn lvn_fn(function: &mut Function) -> bool {
    let mut changed: bool = false;
    let mut bbs = function.get_basic_blocks();

    for bb in bbs.iter_mut() {
        let bb_changed = lvn_bb(bb);
        changed |= bb_changed;
    }

    if changed {
        // bb has changed, flush bb's instrs back to function
        function.instrs.clear();
        for basic_block in bbs.iter_mut() {
            function.instrs.append(&mut basic_block.instrs); // append is okay because bbs are
                                                             // never used again
        }
    }
    changed
}

pub fn lvn_pass(program: &mut Program) -> bool {
    let mut changed: bool = false;

    for function in program.functions.iter_mut() {
        changed |= lvn_fn(function);
    }

    changed
}
