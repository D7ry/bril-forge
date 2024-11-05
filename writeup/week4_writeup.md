# Week 4 Writeup: Memory Aliasing Analysis

All optimization passes are verified with bril programs in `benchmarks` to ensure correctness.

## Memory Alias Analysis

Implementation details: [pointer_analysis.rs](../src/passes/pointer_analysis.rs)

We implement a simple, intra-procedural aliasing analysis using the dataflow analysis technique to
build a point-to graph that maps variable name to memory allocations.

Each memory allocation is given an index identical to the instruction's absolute index in the
function.

### Point-To Graph

```rust
let point_to_graph: &mut HashMap<String, HashSet<usize>> = HashMap::new();
```
The point-to data structure allows for checking aliasing of two pointer variables by checking the
intersections between two sets:

```rust
fn var_alias(var1: &String, var2: &String, point_to_graph: &HashMap<String, HashSet<usize>>) -> bool{
    if point_to_graph.contains_key(var1) && point_to_graph.contains_key(var2) {
        let var1_pointed_to: &HashSet<usize> = point_to_graph.get(var1).unwrap();
        let var2_pointed_to: &HashSet<usize> = point_to_graph.get(var2).unwrap();

        let mut has_alias: bool = false;
        // check for point to graph overlap
        for v in var1_pointed_to.iter() {
            if var2_pointed_to.contains(v) {
                has_alias = true;
                break;
            }
        }
        has_alias
    } else {
        false
    }
}
```

Building the point-to graph involves interating through instructions and populating the structure:

``` rust
fn build_point_to_graph(
    bb: &BasicBlock,
    bb_inst_offset: usize,
    point_to_graph: &mut HashMap<String, HashSet<usize>>,
    num_fn_insts: usize,
) -> bool {
    let mut changed: bool = false;

    for (inst_id_local, inst) in bb.instrs.iter().enumerate() {
        let inst_id_global: usize = inst_id_local + bb_inst_offset; // function-scope instruction
                                                                    //x = alloc n: x points to this allocations
                                                                    //x = id y: x points to the same locations as y did
                                                                    //x = ptradd p offset: same as id (conservative)
                                                                    //x = load p: we aren't tracking anything about p, so x points to all memory locations
        match inst {
            // why don't we have cpp iterators ugh
            Instruction::Opcode(inst) => match inst {
                OpcodeInstruction::Alloc { args, dest, typ } => {
                    if point_to_graph.contains_key(dest) == false {
                        point_to_graph.insert(dest.clone(), HashSet::new());
                    }
                    let pointed_to: &mut HashSet<usize> = point_to_graph.get_mut(dest).unwrap();
                    pointed_to.insert(inst_id_global);
                    changed |= true;
                }
                OpcodeInstruction::Ptradd { args, dest, typ }
                | OpcodeInstruction::Id { args, dest, typ } => {
                    assert!(args.len() == 1 || args.len() == 2);
                    let src_var_name = args.first().unwrap();
                    let mut src_pointed_to: HashSet<usize> = HashSet::new();
                    if let Some(src_pointed_to_it) = point_to_graph.get(src_var_name) {
                        src_pointed_to = src_pointed_to_it.clone();
                    }

                    if point_to_graph.contains_key(dest) == false {
                        point_to_graph.insert(dest.clone(), HashSet::new());
                    }
                    let pointed_to: &mut HashSet<usize> = point_to_graph.get_mut(dest).unwrap();
                    pointed_to.extend(src_pointed_to);
                    changed |= true;
                }
                OpcodeInstruction::Load { args, dest, typ } => {
                    if point_to_graph.contains_key(dest) == false {
                        point_to_graph.insert(dest.clone(), HashSet::new());
                    }
                    let pointed_to: &mut HashSet<usize> = point_to_graph.get_mut(dest).unwrap();
                    for i in 0..num_fn_insts {
                        // points to everything
                        pointed_to.insert(i);
                    }
                    changed |= true;
                }
                _ => {}
            },
            _ => {}
        }
    }

    changed
}
```

## Dead-Store Elimination

With the point-to graph, we can perform a naive dead-store elimination akin to the one used in naive
DCE, where we treat `load` as variable ues and `store` as variable assignment.

`load` instruction counts as using the target memory address as well as all memories that aliases
with the target -- this is where the point-to graph comes in.

```rust
fn dead_store_elimination_bb(
    bb: &mut BasicBlock,
    point_to_graph: &HashMap<String, HashSet<usize>>, // var name -> memory ids var could point to
) -> bool {
    let mut insts_to_delete: Vec<usize> = Vec::new();

    let mut unused_stores: HashMap<String, usize> = HashMap::new(); // <store dst, inst idx>
    // going through instructions in order
    for (inst_idx, inst) in bb.instrs.iter().enumerate() {
        if let Some(result) = inst.get_result() {
            unused_stores.remove(&result);
        }
        match inst {
            Instruction::Opcode(inst) => match inst {
                OpcodeInstruction::Store { args } => {
                    // if any previous stores to the same location remains unused, remove
                    // everything.
                    assert!(args.len() == 2);
                    // store, location, value
                    let store_dst = args.get(0).unwrap();
                    if let Some(unused_store_inst_idx) = unused_stores.get(store_dst) {
                        insts_to_delete.push(unused_store_inst_idx.clone());
                    }
                    unused_stores.insert(store_dst.clone(), inst_idx);
                }
                OpcodeInstruction::Load { args, dest, typ } => {
                    // if anything loads from the location, it's used!
                    assert!(args.len() == 1);
                    // for all unused stores, check for aliasing with the src of this load,
                    // if they alias, the unused store should be flagged as used.
                    let load_src = args.first().unwrap();
                    let mut used_stores: Vec<String> = Vec::new();
                    for elem in unused_stores.iter() {
                        let store_dst = elem.0;
                        let _store_inst_idx = elem.1.clone();
                        if var_alias(store_dst, load_src, point_to_graph) {
                            used_stores.push(store_dst.clone());
                        }
                    }
                    for store in used_stores {
                        unused_stores.remove(&store);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    let changed: bool = !insts_to_delete.is_empty();

    // pop insts in reverse order
    insts_to_delete.sort();
    insts_to_delete.reverse();

    for idx in insts_to_delete.iter() {
        bb.instrs.remove(*idx);
    }

    changed
}
```

The trivial dead-store did not demonstrate much difference when run on the test benchmark suite, the
difference is better shown with the following tests:

before:
```
@simple_dead_store {
  c1: int = const 1;
  v0: ptr<int> = alloc c1;
  x1: int = const 3;

  store v0 x1; # store x1 into v0(dead store)
  x2: int = const 4;
  store v0 x2; # store x2 into v0
  
  x3: int = load v0;
  print x3;
  free v0;
}
```

after:
```
@simple_dead_store {
  c1: int = const 1;
  v0: ptr<int> = alloc c1;
  x1: int = const 3;

  x2: int = const 4;
  store v0 x2;

  x3: int = load v0;
  print x3;
  free v0;
}
```

after DCE and Store-Load forwarding
```
@simple_dead_store {
  c1: int = const 1;
  v0: ptr<int> = alloc c1;

  x2: int = const 4;

  x3: int = id x2;
  print x3;
  free v0;
}
```

after constant prop and another round of DCE:
```
@simple_dead_store {
  x3: int = const 4;
  print x3;
}
```

One can observe that dead-store elimination opens up optimization opportunities for other optimization passes,
this is due to the pattern that `store` in the current DCE implementation is always considered as an `effective`
instruction that flags a variable as live.


### Future Work: Global-DCE like dead-store elimination

A Global-DCE like dead-store elimination can also be implemented, using liveness analysis similarly,
in [live.rs](../src/passes/live.rs). When we're populating liveness table, however, we should
mark all pointers that alias with a pointer to be alive.
