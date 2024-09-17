# Week1 Writeup

All optimization passes verified with bril programs in `benchmarks` to ensure correctness.

## Naive DCE

Naive DCE is the most naive DCE(duh) to implement. Each run processes one
function, as opposed to BB-based DCE. The algorithm is simple:
1. go through the function, generate a use list of all variables ever used
2. go through the function again, for each instruction, if the instruction's 
result variable is not in the use list, and the instruction does not have any other 
side effects, the instruction can be safely deleted.

For Bril specifically, `Print`, `Call`, `Ret`, `Store`, and `Free` are instructions
that may have side effects.

While naive DCE isn't very effective given its performance cost, it does surprisingly well
in a few cases such as for `eight-queens`, where 5% of instructions are optimized away at run time.
That is a lot of physical time for a javascript program.

### Results

The following are some of the more significant results from all benchmarks:
|benchmark|run|result|
|----|----|----|
|hanoi|baseline|99|
|hanoi|naive_dce_pass|84|
|fact|baseline|229|
|fact|naive_dce_pass|228|
|fizz-buzz|baseline|3652|
|fizz-buzz|naive_dce_pass|3552|
|check-primes|baseline|8468|
|check-primes|naive_dce_pass|8419|
|pascals-row|baseline|146|
|pascals-row|naive_dce_pass|139|
|armstrong|baseline|133|
|armstrong|naive_dce_pass|130|
|loopfact|baseline|116|
|loopfact|naive_dce_pass|115|
|cholesky|baseline|3761|
|cholesky|naive_dce_pass|3745|
|mat-inv|baseline|1044|
|mat-inv|naive_dce_pass|1043|
|mandelbrot|baseline|2720947|
|mandelbrot|naive_dce_pass|2720813|
|eight-queens|baseline|1006454|
|eight-queens|naive_dce_pass|959702|
|sieve|baseline|3482|
|sieve|naive_dce_pass|3428|
|binary-search|baseline|78|
|binary-search|naive_dce_pass|75|
|quicksort|baseline|264|
|quicksort|naive_dce_pass|256|
|two-sum|baseline|98|
|two-sum|naive_dce_pass|88|
|csrmv|baseline|121202|
|csrmv|naive_dce_pass|120644|

## Trivial Local DCE -- dead store

The trivial local DCE operates on a BB level -- meaning it can assume that all instructions
the DCE pass is running on are guaranteed to be invoked in a sequential order. This gives some
additional optimization opportunities.

The reasoning behind trivial local DCE is also simple: it iterates over instructions in a BB
in a sequential order, and looks for instructions that overwrites results of previous instructions.
If the previou instructions' results aren't used anywhere in-between, the previous instruction is
then considered a dead-store.

Note that since we are yet to implement data flow analysis, the previous instruction also needs to
have no side-effect, before the dead store elimination happens. For example, the result of a
CallInst can be subject to dead-store, but one cannot make assumption to the CallInst.

### BB Generation

The following algorithm is used to generate basic blocks:
```rust

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
                current_block.in_label = inst.get_result(); // mark in label
            }
            (_, true) => {
                // is control
                // push control inst to current block
                current_block.instrs.push(inst.clone());
                ret.push(current_block);
                // end current block
                current_block = BasicBlock::new();
            }
            (true, true) => {
                panic!("instruction cannot be both a label and a control instruction!");
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
```


### Results

The result is rather underwhelming. Local dead-store is an extremely rare case -- there's nearly no
observed runtime instruction reduction from the official tests. However, for a trivial case as the
following, local dead-store managed to eliminate all the dead stores.
```bril
@main() {
    va: int = const 1;
    va: int = const 2; # dead
    va: int = const 3; # dead
    va: int = const 4; # dead
    print va;
}
```

However, for cases that are slightly more complex, local dead-store is not sufficient:
```bril
@main() {
    va: int = const 1; # dead, but cannot eliminate
    vb: int = const 2;
.done
    print vb;
}
```
For the above case, `va` is clearly a result of dead-store, but the pass cannot make assumption to
the use chain of `va` (as there's no use chain). Need a defuse chain to optimize away it.

## Local Value Numbering

Local value numbering allows us to give each evaluation of expressions a unique id.

A interesting challenge from the current LVN implementation comes from the lack of SSA:
values can get invalidated overtime as we iterate through the Insts in a BB.

An example would be:
```cpp
int a = 0;
int b = 1;
int c = a + b; // c's value number is hashed to add(a,b)
a++;
int d = a + b; // cannot apply CSE here because the expression's operand gets updated
```

While SSA can solve this, we can also maintain an hashmap of `<variable, Vec<ValueNumber>`,
and invalidate the value numbers over time as `variable` are updated -- i.e. whenever `variable`
is the lvalue of an assign stmt. _Because rust compiler loves complaining about anything related to
mutation I decided to use this trick and it works_

### Results

The current implementation of LVN can perform CSE on local BBs. It also respects commutativity by 
sorting the operands in commutative arithmetics before hasing _(should have used XOR, but due to my
limited rust knowledge the compiler complained for 15 min straight and I gave up TT)_

A simple example test case:

before:
```bril
@main() {
    va: int = const 1;
    vb: int = const 2;
    vc: int = add va vb;
    vd: int = add vb va;
    print va;
}
```
after:
```bril
@main() {
    va: int = const 1;
    vb: int = const 2;
    vc: int = add va vb;
    vd: int = id vc;
    print va;
}
```
