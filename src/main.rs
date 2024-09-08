use std::env;
use std::collections::HashMap;
mod ast;
mod passes;

use ast::*;

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
    use passes::*;
    let dispatch_table : HashMap<&str, fn(&mut Program) -> bool>  = create_pass_map!(
        delete_everything_pass,
        do_nothing_pass,
        naive_dce_pass,
        local_dce_pass
    );

    // read program
    let mut program: Program = ast::read_from_pipe();

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

    // can call program.dump() similar to llvm to debug

    program.dump_json(); // json is piped out to the output
}
