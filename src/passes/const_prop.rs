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
                    // constant values gets recoreded into the value table
                    OpcodeInstruction::Const { dest, typ, value } => {
                        // TODO: add const prop for other types
                        if value.is_number() {
                            ctx.constant_values.insert(dest.clone(), value.clone());
                            // println!("{} is constant: {}", dest, value);
                        }
                    }
                    // none-const values, when re-assigned, gets removed from value table.
                    _ => {
                        // remove existing constants if the inst changes value
                        // TODO: can we do better here? maybe some instructions do self-assignment, or + 0
                        // maybe this is better handled with LVN?
                        if let Some(dest) = opcode_inst.get_dest() {
                            // println!("removing {} as constant due to re-assignment", dest);
                            ctx.constant_values.remove(&dest);
                        }
                    }
                }

                // replace inst variable uses with constants
                if let Some(args) = opcode_inst.get_args() {
                    let mut all_args_constants: bool = true;
                    let mut arg_values: Vec<&serde_json::Value> = Vec::new();
                    for arg in args.iter_mut() {
                        if let Some(value) = ctx.constant_values.get(arg) {
                            arg_values.push(value);
                        } else {
                            all_args_constants = false;
                            break;
                        }
                    }
                    // if all args are constants, replace inst with an const inst
                    if all_args_constants {
                        let evaluated_const_value: Option<serde_json::Value>;
                        match opcode_inst {
                            OpcodeInstruction::Add { .. } => {
                                let mut const_value: i64 = 0;
                                // accumulate arg values
                                for val in arg_values {
                                    const_value += val.as_i64().unwrap();
                                }
                                evaluated_const_value = Some(const_value.into());
                            }
                            _ => evaluated_const_value = Option::None,
                        }
                        if let Some(value) = evaluated_const_value {
                            if let Some(dest) = opcode_inst.get_dest() {
                                if let Some(typ) = opcode_inst.get_type() {
                                    // populate const table with new const
                                    ctx.constant_values.insert(dest.clone(), value.clone());

                                    // construct new const value
                                    let const_inst = OpcodeInstruction::Const { dest, typ, value };
                                    // write back
                                    *opcode_inst = const_inst;
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    (changed, ctx)
}

// constant propagation that operates on a function scope
fn fn_constant_prop(function: &mut Function) -> bool {
    let empty_state: ConstantState = ConstantState {
        constant_values: HashMap::new(),
    };

    let mut changed: bool = false;

    let mut bbs = function.get_basic_blocks();
    for bb in bbs.iter_mut() {
        let local_constant_prop_res = local_constant_prop(bb, &empty_state);
        if local_constant_prop_res.0 {
            changed = true;
        }
    }

    // bb has changed, flush bb's instrs back to function
    function.instrs.clear();
    for basic_block in bbs.iter_mut() {
        // note here we move all instrs in bb back to function, bbs
        // are unusable from this point on.
        function.instrs.append(&mut basic_block.instrs);
    }

    changed
}

pub fn global_const_propagation_pass(program: &mut Program) -> bool {
    let mut changed: bool = false;
    for function in program.functions.iter_mut() {
        changed |= fn_constant_prop(function);
    }

    changed
}
