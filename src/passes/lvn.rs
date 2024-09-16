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
    // <variable, Vec<value numbers depending on the variable>>
    let mut variable_to_value_numbers: HashMap<String, Vec<u32>> = HashMap::new();
    let mut value_number_to_expression: HashMap<u32, String> = HashMap::new();
    let mut value_number_to_variable: HashMap<u32, String> = HashMap::new();

    // build up the lvn table, good old spir-v time
    let mut vn: u32 = 0; // counter for value number

    //
    let mut inst_to_replace: Vec<(usize, OpcodeInstruction)> = Vec::new();

    // note that we invalidate value numbers whose expression's operands are updated
    for (inst_idx, inst) in bb.instrs.iter_mut().enumerate() {
        match inst {
            // only opcode insts can have rhs
            Instruction::Opcode(opcode_inst) => {
                // please don't look at it
                if let Some(rhs_expr_hash) = get_rhs_hash(opcode_inst) {
                    if let Some(opcode_inst_dest) = opcode_inst.get_dest() {
                        if let Some(opcode_inst_type) = opcode_inst.get_type() {
                            // found matching value member, can perform CSE
                            if expression_hash_to_value_number.contains_key(&rhs_expr_hash) {
                                let value_number =
                                    expression_hash_to_value_number.get(&rhs_expr_hash).unwrap();
                                // println!("{} maps to value number {}", rhs_expr_hash, value_number);
                                // CSE
                                let variable =
                                    value_number_to_variable.get(value_number).unwrap().clone();
                                // safe to unwrap here, trust me bro
                                // can replace inst with an assignment
                                let assignment_inst = OpcodeInstruction::Id {
                                    args: vec![variable],
                                    dest: opcode_inst_dest.clone(),
                                    typ: opcode_inst_type,
                                };
                                inst_to_replace.push((inst_idx, assignment_inst));
                                // println!("{:?}", inst_to_replace);
                            } else {
                                // expression not yet stored, store it as value number
                                expression_hash_to_value_number.insert(rhs_expr_hash.clone(), vn);
                                value_number_to_expression.insert(vn, rhs_expr_hash.clone());

                                // record value number dependency on BB variables
                                opcode_inst.get_use_list().iter().for_each(|u| {
                                    if !variable_to_value_numbers.contains_key(u) {
                                        variable_to_value_numbers.insert(u.clone(), Vec::new());
                                    }
                                    variable_to_value_numbers.get_mut(u).unwrap().push(vn)
                                });
                                value_number_to_variable.insert(vn, opcode_inst_dest.clone());
                                vn += 1;
                            }
                        }
                        // invalidate all value numbers that depends on the lvalue of this assign
                        // stmt
                        if let Some(value_numbers_to_invalidate) =
                            variable_to_value_numbers.get_mut(&opcode_inst_dest)
                        {
                            // println!("Invalidating all value numbers depending on {}", opcode_inst_dest);
                            for value_number in value_numbers_to_invalidate.iter() {
                                // clear off existence of the value number, it's not usable anymore
                                if let Some(expression_hash) =
                                    value_number_to_expression.remove(value_number)
                                {
                                    expression_hash_to_value_number.remove(&expression_hash);
                                }
                                // technically since we erase the expression hash,
                                // we'll never hit the invalid value number and use it in
                                // value_number_to_variable to perform CSE,
                                // but just to be safe we eliminate it altogether
                                value_number_to_variable.remove(value_number);
                            }
                            value_numbers_to_invalidate.clear();
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
