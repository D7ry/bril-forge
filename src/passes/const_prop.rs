use crate::ast;
use ast::*;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone)]
struct ConstantState {
    constant_values: HashMap<String, serde_json::Value>, // variable identifier -> constant value
}

// perform constant prop on a BB
// var_state: contexual information to the BB
// returns whether constant prop changes anything, and a new constant prop state
fn local_constant_prop(bb: &mut BasicBlock, mut ctx: ConstantState) -> (bool, ConstantState) {
    let mut changed: bool = false;

    // we mutate the constant states as we go through the insts
    for inst in bb.instrs.iter_mut() {
        match inst {
            Instruction::Opcode(opcode_inst) => {
                // update constant states
                // insert new constants
                match opcode_inst {
                    // constant values gets recoreded into the value table
                    OpcodeInstruction::Const { dest, value, .. } => {
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
                                for val in arg_values {
                                    const_value += val.as_i64().unwrap();
                                }
                                evaluated_const_value = Some(const_value.into());
                            }
                            OpcodeInstruction::FAdd { .. } => {
                                let mut const_value: f64 = 0.0;
                                for val in arg_values {
                                    const_value += val.as_f64().unwrap();
                                }
                                evaluated_const_value = Some(const_value.into());
                            }
                            OpcodeInstruction::Mul { .. } => {
                                let mut const_value: i64 = 1;
                                for val in arg_values {
                                    const_value *= val.as_i64().unwrap();
                                }
                                evaluated_const_value = Some(const_value.into());
                            }
                            OpcodeInstruction::FMul { .. } => {
                                let mut const_value: f64 = 1.0;
                                for val in arg_values {
                                    const_value *= val.as_f64().unwrap();
                                }
                                evaluated_const_value = Some(const_value.into());
                            }
                            _ => evaluated_const_value = Option::None,
                        }
                        if let Some(value) = evaluated_const_value {
                            if let Some(dest) = opcode_inst.get_dest() {
                                if let Some(typ) = opcode_inst.get_type() {
                                    changed = true;
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

// join constant states. for a constant value of variable `x` to carry on, the following must
// satisfy:
// 1. all of the bb's parents have a constate state of variable `x`
// 2. all of the bb's parents that have the `x` constant state need to have `x` the same const
//    value.
fn join_constant_states(states: Vec<&ConstantState>) -> ConstantState {
    // println!("Joining constant states of {} states", states.len());
    let mut joined_state: ConstantState = ConstantState {
        constant_values: HashMap::new(),
    };

    // count occurance of constant vals, # occurance must be equal to states.len() i.e.
    // the constant has to exist in all of its parents

    // variables -> <# of occurance in parent states, first occurence's value>
    let mut const_vals: HashMap<String, (usize, serde_json::Value)> = HashMap::new();

    for state in states.iter() {
        for (key, val) in state.constant_values.iter() {
            if let Some(entry) = const_vals.get_mut(key) {
                if entry.1 == *val {
                    entry.0 += 1; // increment counter
                }
            } else {
                const_vals.insert(key.clone(), (1, val.clone()));
            }
        }
    }

    // populate joined_state
    for elem in const_vals {
        // println!("checking const val {}", elem.0);
        if elem.1.0 != states.len() {
            continue;
        }
        // the same const value of such var has occured for states.len() times,
        // and all of values are equal. Can join the state.
        joined_state.constant_values.insert(elem.0, elem.1.1);
    }

    joined_state
}

// constant propagation that operates on a function scope
fn fn_constant_prop(function: &mut Function) -> bool {
    let mut changed: bool = false;
    let mut bbs = function.get_basic_blocks();

    // each index corresponds to one bb
    let mut bb_consts_info: Vec<ConstantState> = Vec::new();
    bb_consts_info.resize(
        bbs.len(),
        ConstantState {
            constant_values: HashMap::new(),
        },
    );

    // worklist of bb indices
    let mut work_list: VecDeque<usize> = VecDeque::new();
    let mut in_work_list: HashSet<usize> = HashSet::new(); // indices already in worklist to
                                                           // prevent repetition

    for i in 0..bbs.len() {
        work_list.push_back(i);
        in_work_list.insert(i);
    }

    // iterate until convergence.
    while let Some(bb_idx) = work_list.pop_front() {
        in_work_list.remove(&bb_idx); // no longer in worklist
                                      //
        let bb = bbs.get_mut(bb_idx).unwrap();
        // join all of bb's parents' constant state to figure out bb's initial state
        let mut parent_states: Vec<&ConstantState> = Vec::new();
        for parent_idx in bb.in_bb_indices.iter() {
            parent_states.push(bb_consts_info.get(*parent_idx).unwrap());
        }
        // println!("parent states: {}", parent_states.len());
        let joined_state = join_constant_states(parent_states);
        let local_constant_prop_res = local_constant_prop(bb, joined_state);
        // update constant state
        // here, we update constant state no matter whether there is change.
        // TODO: add const state hashing, this serves a better indicator than "changed"
        // flag of local const prop call, which is what we really care about. this will
        // allow state changes from one bb's parents to be propagated to its children,
        // whether if the bb's instr changes.
        let const_state = bb_consts_info.get_mut(bb_idx).unwrap();
        *const_state = local_constant_prop_res.1;
        if local_constant_prop_res.0 == true {
            // changed
            changed = true;
            // push all successors of this bb back to the worklist
            for successor in bb.out_bb_indices.iter() {
                if !in_work_list.contains(successor) {
                    in_work_list.insert(*successor);
                    work_list.push_back(*successor);
                }
            }
        }
    }

    if changed {
        function.update(bbs);
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
