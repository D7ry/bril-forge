# Week1 Writeup

All optimization passes are verified with bril programs in `benchmarks` to ensure correctness.

## Global Constant Prop & Folding

Implementation source: `constant_prop.rs`

One interesting detail about bril is the lack of constant to arithmetic operations i.e. all operands
to arithmetic operations are variables. This forces us to couple constant prop and folding into one pass,
as we need the constant context from constant prop to perform constant folding -- had another design
idea of iteratively alternating between separate constant prop and folding pass. 

One use case of solo constant folding exists for programs that tend to have many expressions whose
operands are constants to begin with, and whose results aren't getting used as input value for
another expression that assigns to a result. In this (albeit rare) case, one constant folding pass 
could effectively transform the program.


### Local Constant Prop

We define `state` as the information regarding a constant prop/constant folding operation. The
implementation, on a basic-block level, can be summed up as:

```rust
fn const_prop(bb, state):
    for inst in bb:
        if inst.is_const():
            state.const_tbl[inst.dst] = inst.arg;
        else:
            state.const_tbl.remove(inst.dst);
        if inst.is_expression():
            if state.const_tbl.contains_all(inst.args):
                const_eval(inst.opcode, state.const_tbl, inst.args)
                state.const_tbl[inst.dst] = inst.arg;
```

We populate the const table as we iterate through the instrs in bb, and performs const eval on
expressions whose arguments are all known. We also put the evaluated const into the const table
for further propagation.

Result of local constant prop:

before:
```
@main() {
    va: int = const 1;
    vb: int = const 2;
    vc: int = add va vb; # va, vb known at compile-time
    print vc;
}
```

after:
```
@main() {
    va: int = const 1;
    vb: int = const 2;
    vc: int = const 3; # vc has been evaluated
    print vc;
}
```
Constant prop, regardless of being local or global, also works well with DCE. A trivial DCE pass
that checks for use list, could turn the above further into:
```
@main() {
    vc: int = const 3; # vc has been evaluated
    print vc;
}
```

### Global Constant Prop

`state` becomes actually useful in global constant prop. In local constant prop, we trivially 
make the state empty -- no information is available at the beginning of the analysis. For global, 
before calling `const_prop()` on a bb, we join the states passed off from the all of bb's parents.
The join function simply looks at all key-value pairs from both states, and keeps the pairs where
both key and value are identical -- the constant values are the same from both parents to the bb.

For global constant prop, we also iterate until convergence; to keep track of the state changes,
`fn const_prop(bb, state)` now returns a tuple of `bool, state` -- indicating whether the state has
been modified, and a copy of the new state at the end of the constant prop. This allows constant
states to be propagated across multiple basic blocks.

By the end of calling `const_prop()` on a bb, if its constant state changes, `const_prop()` with
the new state is to be evaluated on all of the bb's successors. We use a worklist to keep track of
bbs that needs to be re-const-prop'd, and a table of `<bb, state>` to memoize const states, in case
the re-evaluation of one bb requires constant states of multiple parents.

To expedite BB lookup, basic block construction algorithm has been modified that, for each bb, bb's
in and out labels, as well as indices to the corresponding in and out bbs in the bb array would be
returned. (Had a lot of issues implementing this due to the limited information on bril's invariants,
things like unnamed BBs and such. you really can't take something as elegant as SPIR-V for granted).

The core of global const prop algo: 
```rust
while let Some(bb_idx) = work_list.pop_front() {
    in_work_list.remove(&bb_idx); // no longer in worklist
    let bb = bbs.get_mut(bb_idx).unwrap();
    // join all of bb's parents' constant state to figure out bb's initial state
    let mut parent_states: Vec<&ConstantState> = Vec::new();
    for parent_idx in bb.in_bb_indices.iter() {
        parent_states.push(bb_consts_info.get(*parent_idx).unwrap());
    }
    let joined_state = join_constant_states(parent_states);
    let local_constant_prop_res = local_constant_prop(bb, joined_state);
    if local_constant_prop_res.0 == true {
        // changed
        changed = true;
        // update constant state
        let const_state = bb_consts_info.get_mut(bb_idx).unwrap();
        *const_state = local_constant_prop_res.1;
        // push all successors of this bb back to the worklist
        for successor in bb.out_bb_indices.iter() {
            if !in_work_list.contains(successor) {
                in_work_list.insert(*successor);
                work_list.push_back(*successor);
            }
        }
    }
}
```

#### Global Const Prop and LVN

Global constant prop cannot entirely replace LVN thanks to LVN's CSE effect. CSE optimizes away
redundant insts where some variables aren't known to be sure to be constant; take the following as
an example:

```c
int i = 0;
int j = 1;
if (some_cond) {
    j = 2;
}
int k = i + j;
int l = i + j;
```

`i + j` is a common subexpression used by both `k` and `l`, but the states aren't strong enough to
guarantee its constantness -- CSE, however, would optimize `l` into `int l = k`

## Liveness Analysis & Global DCE
