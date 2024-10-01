use crate::ast;
use ast::*;
use std::collections::HashMap;

#[derive(Clone)]
struct ConstantState {
    constant_values: HashMap<String, serde_json::Value>, // variable identifier -> constant value
}

// perform constant prop on a BB
// var_state: contexual information to the BB
// returns whether constant prop changes anything, and a new constant prop state
fn local_constant_prop(bb: &mut BasicBlock, ctx: &ConstantState) -> (bool, ConstantState) {
    let mut changed: bool = false;
    let mut ctx = ctx.clone();

    // we mutate the constant states as we go through the insts
    for inst in bb.instrs.iter_mut() {
        match inst {
            Instruction::Opcode(opcode_inst) => {
                // update constant states
                // insert new constants
                match opcode_inst {
                    OpcodeInstruction::Const { dest, typ, value } => {
                        // TODO: add const prop for other types
                        if value.is_number() {
                            ctx.constant_values.insert(dest.clone(), value.clone());
                        }
                    }
                    _ => {}
                }
                // remove existing constants if the inst changes value
                // TODO: can we do better here? maybe some instructions do self-assignment, or + 0
                // maybe this is better handled with LVN?
                if let Some(dest) = opcode_inst.get_dest() {
                    ctx.constant_values.remove(&dest);
                }

                // replace inst variable uses with constants
                if let Some(args) = opcode_inst.get_args() {
                    // if all args are constants, replace inst with an const inst
                    for arg in args.iter_mut() {
                        if let Some(const_val) = ctx.constant_values.get(arg) {
                            // FIXME: implement
                            changed = true;
                        }
                    }
                    // dest of inst is also a constant

                }
            }
            _ => {}
        }
    }

    (changed, ctx)
}


// constant propagation that operates on a function scope
fn fn_constant_prop(function: &mut Function) -> bool {
    let mut changed: bool = false;

    let mut bbs = function.get_basic_blocks();
    for bb in bbs.iter_mut() {}

    changed
}

pub fn global_const_propagation_pass(program: &mut Program) -> bool {
    let mut changed: bool = false;
    for function in program.functions.iter_mut() {
        changed |= fn_constant_prop(function);
    }

    changed
}
