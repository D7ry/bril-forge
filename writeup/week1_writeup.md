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

|benchmark|run|result|
|----|----|----|
|dead-branch|baseline|1196|
|dead-branch|naive_dce_pass|1196|
|function_call|baseline|timeout|
|function_call|naive_dce_pass|timeout|
|hanoi|baseline|99|
|hanoi|naive_dce_pass|84|
|fact|baseline|229|
|fact|naive_dce_pass|228|
|reverse|baseline|46|
|reverse|naive_dce_pass|46|
|palindrome|baseline|298|
|palindrome|naive_dce_pass|298|
|collatz|baseline|169|
|collatz|naive_dce_pass|168|
|relative-primes|baseline|1923|
|relative-primes|naive_dce_pass|1914|
|euclid|baseline|563|
|euclid|naive_dce_pass|562|
|up-arrow|baseline|252|
|up-arrow|naive_dce_pass|252|
|perfect|baseline|232|
|perfect|naive_dce_pass|232|
|lcm|baseline|2326|
|lcm|naive_dce_pass|2326|
|quadratic|baseline|785|
|quadratic|naive_dce_pass|783|
|sum-check|baseline|5018|
|sum-check|naive_dce_pass|5018|
|binary-fmt|baseline|100|
|binary-fmt|naive_dce_pass|100|
|totient|baseline|253|
|totient|naive_dce_pass|253|
|gcd|baseline|46|
|gcd|naive_dce_pass|46|
|birthday|baseline|484|
|birthday|naive_dce_pass|483|
|fizz-buzz|baseline|3652|
|fizz-buzz|naive_dce_pass|3552|
|check-primes|baseline|8468|
|check-primes|naive_dce_pass|8419|
|fitsinside|baseline|10|
|fitsinside|naive_dce_pass|10|
|is-decreasing|baseline|127|
|is-decreasing|naive_dce_pass|127|
|rectangles-area-difference|baseline|14|
|rectangles-area-difference|naive_dce_pass|14|
|pascals-row|baseline|146|
|pascals-row|naive_dce_pass|139|
|orders|baseline|5352|
|orders|naive_dce_pass|5351|
|factors|baseline|72|
|factors|naive_dce_pass|72|
|ackermann|baseline|1464231|
|ackermann|naive_dce_pass|1464231|
|sum-divisors|baseline|159|
|sum-divisors|naive_dce_pass|159|
|bitwise-ops|baseline|1690|
|bitwise-ops|naive_dce_pass|1689|
|sum-sq-diff|baseline|3038|
|sum-sq-diff|naive_dce_pass|3036|
|bitshift|baseline|167|
|bitshift|naive_dce_pass|167|
|sum-bits|baseline|73|
|sum-bits|naive_dce_pass|72|
|armstrong|baseline|133|
|armstrong|naive_dce_pass|130|
|loopfact|baseline|116|
|loopfact|naive_dce_pass|115|
|catalan|baseline|659378|
|catalan|naive_dce_pass|659378|
|digital-root|baseline|247|
|digital-root|naive_dce_pass|247|
|primes-between|baseline|574100|
|primes-between|naive_dce_pass|574100|
|pythagorean_triple|baseline|61518|
|pythagorean_triple|naive_dce_pass|61518|
|recfact|baseline|104|
|recfact|naive_dce_pass|103|
|mod_inv|baseline|558|
|mod_inv|naive_dce_pass|556|
|cholesky|baseline|3761|
|cholesky|naive_dce_pass|3745|
|mat-inv|baseline|1044|
|mat-inv|naive_dce_pass|1043|
|n_root|baseline|733|
|n_root|naive_dce_pass|733|
|conjugate-gradient|baseline|1999|
|conjugate-gradient|naive_dce_pass|1998|
|euler|baseline|1908|
|euler|naive_dce_pass|1907|
|riemann|baseline|298|
|riemann|naive_dce_pass|298|
|pow|baseline|36|
|pow|naive_dce_pass|34|
|ray-sphere-intersection|baseline|142|
|ray-sphere-intersection|naive_dce_pass|142|
|leibniz|baseline|12499997|
|leibniz|naive_dce_pass|12499997|
|sqrt|baseline|322|
|sqrt|naive_dce_pass|321|
|newton|baseline|217|
|newton|naive_dce_pass|217|
|cordic|baseline|517|
|cordic|naive_dce_pass|516|
|mandelbrot|baseline|2720947|
|mandelbrot|naive_dce_pass|2720813|
|norm|baseline|505|
|norm|naive_dce_pass|505|
|eight-queens|baseline|1006454|
|eight-queens|naive_dce_pass|959702|
|sieve|baseline|3482|
|sieve|naive_dce_pass|3428|
|binary-search|baseline|78|
|binary-search|naive_dce_pass|75|
|adler32|baseline|6851|
|adler32|naive_dce_pass|6850|
|mat-mul|baseline|1990407|
|mat-mul|naive_dce_pass|1990402|
|quicksort-hoare|baseline|27333|
|quicksort-hoare|naive_dce_pass|27783|
|vsmul|baseline|86036|
|vsmul|naive_dce_pass|86036|
|bubblesort|baseline|253|
|bubblesort|naive_dce_pass|242|
|max-subarray|baseline|193|
|max-subarray|naive_dce_pass|193|
|major-elm|baseline|47|
|major-elm|naive_dce_pass|47|
|quicksort|baseline|264|
|quicksort|naive_dce_pass|256|
|dot-product|baseline|88|
|dot-product|naive_dce_pass|88|
|two-sum|baseline|98|
|two-sum|naive_dce_pass|88|
|csrmv|baseline|121202|
|csrmv|naive_dce_pass|120644|
|fib|baseline|121|
|fib|naive_dce_pass|120|
|adj2csr|baseline|56629|
|adj2csr|naive_dce_pass|56625|
|primitive-root|baseline|11029|
|primitive-root|naive_dce_pass|11024|
|quickselect|baseline|279|
|quickselect|naive_dce_pass|279|

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
