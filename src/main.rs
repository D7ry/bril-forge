use std::collections::HashMap;
use std::env;
mod ast;
mod dom;
mod passes;
use ast::*; // dispatch table definition

macro_rules! create_pass_map {
    ($($name:ident),*) => {
        {
            let mut map = HashMap::new();
            $(
                map.insert(stringify!($name), $name as fn(&mut Program) -> bool);
            )*
            map
        }
    };
}

//
// bril_forge <pass name>...
//
fn main() {
    // construct dispatch table
    use passes::const_prop::*;
    use passes::dce::*;
    use passes::example::*;
    use passes::lvn::*;
    use passes::live::global_dce_pass_using_livenss;
    use passes::loops::*;

    let dispatch_table: HashMap<&str, fn(&mut Program) -> bool> = create_pass_map!(
        // example passes
        delete_everything_pass,
        do_nothing_pass,
        // dce passes
        naive_dce_pass,
        local_dce_pass,
        // lvn pass
        lvn_pass,
        global_const_propagation_pass,
        global_dce_pass_using_livenss,
        loop_invariant_code_motion_pass
    );

    // read program
    let mut program: Program = ast::read_from_pipe();

    // program.dump();
    // println!("after: \n");
    // dispatch passes as specified from stdin
    for arg in env::args().skip(1) {
        let pass = dispatch_table.get(&*arg);
        match pass {
            Some(pass) => {
                let fn_ptr: fn(&mut Program) -> bool = *pass;
                let _res = fn_ptr(&mut program);
            }
            None => {
                panic!("blowing up the program, pass {} does not exist.", arg);
            }
        }
    }

    // program.dump();

    program.dump_json(); // json is piped out to the output
}
