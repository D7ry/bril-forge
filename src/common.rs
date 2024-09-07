use std::io::{self, Read, Write};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// reads a program from a pipe, if not successful panik
pub fn read_from_pipe() -> Program {
    let mut buffer = String::new();
    let _ = io::stdin().read_to_string(&mut buffer).unwrap();
    let program : Program = serde_json::from_str(&buffer).unwrap();
    return program;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Program {
    pub functions: Vec<Function>,
}

impl Program {
    pub fn dump(&self) {
        for function in &self.functions {
            println!("Function: {}", function.name);
            for (i, instr) in function.instrs.iter().enumerate() {
                println!("  Instruction {}: {:?}", i, instr);
            }
        }
    }
    // write json to stdout or blow up the program
    pub fn dump_json_to_stdout(&self) {
        let json = serde_json::to_string(&self).unwrap();
        io::stdout().write_all(json.as_bytes()).unwrap();
        io::stdout().flush().unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<Argument>>,
    pub instrs: Vec<Instruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub return_type: Option<Type>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Argument {
    name: String,
    #[serde(rename = "type")]
    arg_type: Type,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Type {
    Primitive (String),
    Pointer { ptr: String },
    // other wrapper types?
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Instruction { // instruction can either have opcode, or just be label or nop
    Opcode(OpcodeInstruction),
    Label { label: String },
    Nop { op: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op")]
 #[serde(rename_all = "lowercase")]
pub enum OpcodeInstruction {
    #[serde(rename = "const")]
    Const {
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
        value: Value,
    },
    Alloc {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Call {
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dest: Option<String>,
        funcs: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "type")]
        typ: Option<Type>,
    },
    Print {
        args: Vec<String>,
    },
    Free {
        args: Vec<String>,
    },
    Ret {
        args: Vec<String>,
    },
    Id {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Store {
        args: Vec<String>,
    },
    Ptradd {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Br {
        args: Vec<String>,
        labels: Vec<String>,
    },
    Or {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Add {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Sub {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Div {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Mul {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FAdd {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FSub {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FDiv {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FMul {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Eq {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Gt {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Ge {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Lt {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Le {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FEq {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FGt {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FGe {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FLt {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    FLe {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    And {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Not {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Load {
        args: Vec<String>,
        dest: String,
        #[serde(rename = "type")]
        typ: Type,
    },
    Jmp {
        labels: Vec<String>
    }
}
