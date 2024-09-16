// local value numbering
use crate::ast;
use ast::*;
use std::collections::{HashMap, HashSet};

fn hash_expr(args: &Vec<String>, typ: &Type, opcode: &str) -> String {
    let mut hash: String = String::new();
    // opcode
    hash.push_str(&opcode);
    // type
    hash.push_str(&typ.to_string());

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

fn get_rhs_hash(opcode_inst: &OpcodeInstruction) -> Option<String> {
    match opcode_inst {
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
    }
}

// perform lvn on bb that does the following optimizations:
// CSE
//
// Note the pass doens't assume SSA
fn lvn_bb(bb: &mut BasicBlock) -> bool {
    let changed: bool;

    let mut expression_hash_to_value_number: HashMap<String, u32> = HashMap::new();
    let mut variable_to_value_number: HashMap<String, u32> = HashMap::new();
    let mut value_number_to_expression: HashMap<u32, String> = HashMap::new();
    let mut value_number_to_variable: HashMap<u32, String> = HashMap::new();

    // build up the lvn table, good old spir-v time
    let mut i: u32 = 0; // counter for value number

    //
    let mut inst_to_replace: Vec<(usize, OpcodeInstruction)> = Vec::new();

    for (inst_idx, inst) in bb.instrs.iter_mut().enumerate() {
        match inst {
            // only opcode insts can have rhs
            Instruction::Opcode(opcode_inst) => {
                if let Some(rhs_expr_hash) = get_rhs_hash(opcode_inst) {
                    if let Some(opcode_inst_dest) = opcode_inst.get_dest() {
                        if let Some(opcode_inst_type) = opcode_inst.get_type() {
                            if expression_hash_to_value_number.contains_key(&rhs_expr_hash) {
                                let value_number =
                                    expression_hash_to_value_number.get(&rhs_expr_hash).unwrap();
                                // CSE
                                let variable =
                                    value_number_to_variable.get(value_number).unwrap().clone();
                                // safe to unwrap here, trust me bro
                                // can replace inst with an assignment
                                let assignment_inst = OpcodeInstruction::Id {
                                    args: vec![variable],
                                    dest: opcode_inst_dest,
                                    typ: opcode_inst_type,
                                };
                                inst_to_replace.push((inst_idx, assignment_inst));
                            } else {
                                // expression not yet stored, store it as value number
                                expression_hash_to_value_number.insert(rhs_expr_hash.clone(), i);
                                value_number_to_expression.insert(i, rhs_expr_hash.clone());

                                variable_to_value_number.insert(opcode_inst_dest.clone(), i);
                                value_number_to_variable.insert(i, opcode_inst_dest.clone());
                                i += 1;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    changed = !inst_to_replace.is_empty();

    for (idx, opcode_instruction) in inst_to_replace {
        bb.instrs[idx] = ast::Instruction::Opcode(opcode_instruction);
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
