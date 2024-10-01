# Week1 Writeup

All optimization passes verified with bril programs in `benchmarks` to ensure correctness.

## Global Constant Prop & Folding

One interesting detail about bril is the lack of constant to arithmetic operations i.e. all operands
to arithmetic operations are variables. This forces us to couple constant prop and folding into one pass,
as we need the constant context from constant prop to perform constant folding -- had another design
idea of iteratively alternating between separate constant prop and folding pass. 

One use case of solo constant folding exists for programs that tend to have many expressions whose
operands are constants to begin with, and whose results aren't getting used as input value for
another expression that assigns to a result. In this (albeit rare) case, one constant folding pass 
could effectively transform the program.


## Liveness Analysis & Global DCE
