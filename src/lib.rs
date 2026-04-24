use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug)]
enum Value {
    Int(i32),
}

#[derive(Clone, Copy, Debug)]
enum PrimitiveId {
    Dup,
    Add,
    Mul,
    Dot,
    DotS,
    Clear,
}

#[derive(Clone, Debug)]
enum Word {
    Primitive(PrimitiveId),
    User(Vec<Op>),
}

#[derive(Clone, Debug)]
struct Op {
    label: String,
    kind: OpKind,
}

#[derive(Clone, Debug)]
enum OpKind {
    PushInt(i32),
    CallPrim(PrimitiveId),
    CallByName(String),
}

#[derive(Clone, Debug)]
struct Pending {
    name: Option<String>,
    body: Vec<Op>,
}

#[wasm_bindgen]
pub struct Machine {
    stack: Vec<Value>,
    output: Vec<String>,
    history: Vec<String>,
    trace: Vec<String>,
    dictionary: HashMap<String, Word>,
    compiling: Option<Pending>,
}

#[wasm_bindgen]
impl Machine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Machine {
        let mut m = Machine {
            stack: Vec::new(),
            output: vec![
                "Machine created.".to_string(),
                "Supported words: DUP + * . .S CLEAR : ;".to_string(),
            ],
            history: Vec::new(),
            trace: Vec::new(),
            dictionary: HashMap::new(),
            compiling: None,
        };
        m.install_primitives();
        m
    }

    pub fn reset(&mut self) {
        self.stack.clear();
        self.compiling = None;
        self.output.push("VM reset.".to_string());
        self.history.push("--- reset ---".to_string());
        self.trace.push("--- reset ---".to_string());
    }

    pub fn eval_repl(&mut self, line: &str) {
        let line = line.trim();
        if line.is_empty() {
            return;
        }

        self.history.push(format!("> {}", line));
        let tokens: Vec<String> = line.split_whitespace().map(String::from).collect();
        for token in tokens {
            self.dispatch_token(&token);
        }
    }

    pub fn load_source(&mut self, src: &str) {
        let src = src.trim();
        if src.is_empty() {
            return;
        }

        self.history.push("> [load source]".to_string());
        let tokens: Vec<String> = src.split_whitespace().map(String::from).collect();
        for token in tokens {
            self.dispatch_token(&token);
        }
        self.output
            .push(format!("Loaded source ({} chars).", src.len()));
    }

    pub fn get_stack_text(&self) -> String {
        self.stack
            .iter()
            .map(|v| match v {
                Value::Int(n) => n.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn get_output_text(&self) -> String {
        self.output.join("\n")
    }

    pub fn get_history_text(&self) -> String {
        self.history.join("\n")
    }

    pub fn get_dictionary_text(&self) -> String {
        let mut names: Vec<_> = self.dictionary.keys().cloned().collect();
        names.sort();
        names.join("\n")
    }

    pub fn get_trace_text(&self) -> String {
        self.trace.join("\n")
    }
}

impl Machine {
    fn install_primitives(&mut self) {
        self.dictionary
            .insert("DUP".to_string(), Word::Primitive(PrimitiveId::Dup));
        self.dictionary
            .insert("+".to_string(), Word::Primitive(PrimitiveId::Add));
        self.dictionary
            .insert("*".to_string(), Word::Primitive(PrimitiveId::Mul));
        self.dictionary
            .insert(".".to_string(), Word::Primitive(PrimitiveId::Dot));
        self.dictionary
            .insert(".S".to_string(), Word::Primitive(PrimitiveId::DotS));
        self.dictionary
            .insert("CLEAR".to_string(), Word::Primitive(PrimitiveId::Clear));
    }

    fn dispatch_token(&mut self, token: &str) {
        self.do_dispatch(token);
        self.emit_trace(token);
    }

    fn emit_trace(&mut self, token: &str) {
        let mode = if self.compiling.is_some() { "C" } else { "I" };
        let stack = self
            .stack
            .iter()
            .map(|v| match v {
                Value::Int(n) => n.to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ");
        self.trace.push(format!("{} {} | [{}]", mode, token, stack));
    }

    fn do_dispatch(&mut self, token: &str) {
        let upper = token.to_ascii_uppercase();

        if self.compiling.is_some() {
            self.dispatch_compile(token, &upper);
            return;
        }

        if upper == ":" {
            self.compiling = Some(Pending {
                name: None,
                body: Vec::new(),
            });
            return;
        }
        if upper == ";" {
            self.output.push(";: not compiling".to_string());
            return;
        }

        if let Ok(n) = token.parse::<i32>() {
            self.stack.push(Value::Int(n));
            self.output.push(format!("push {}", n));
            return;
        }

        match self.dictionary.get(&upper).cloned() {
            Some(Word::Primitive(prim)) => self.execute_primitive(prim),
            Some(Word::User(ops)) => self.execute_ops(&upper, ops),
            None => self.output.push(format!("unknown token: {}", token)),
        }
    }

    fn dispatch_compile(&mut self, token: &str, upper: &str) {
        let pending = self
            .compiling
            .as_mut()
            .expect("dispatch_compile called without pending");

        if pending.name.is_none() {
            if upper == ":" || upper == ";" {
                self.output
                    .push(format!("compile: expected word name, got {}", token));
                self.compiling = None;
                return;
            }
            pending.name = Some(upper.to_string());
            return;
        }

        if upper == ";" {
            let done = self.compiling.take().unwrap();
            let name = done.name.unwrap();
            let body_len = done.body.len();
            self.dictionary
                .insert(name.clone(), Word::User(done.body));
            self.output
                .push(format!("defined {} ({} tokens)", name, body_len));
            return;
        }

        if upper == ":" {
            self.output
                .push("compile: nested : not allowed".to_string());
            return;
        }

        let op = self.compile_token(token, upper);
        self.compiling.as_mut().unwrap().body.push(op);
    }

    fn compile_token(&self, token: &str, upper: &str) -> Op {
        if let Ok(n) = token.parse::<i32>() {
            return Op {
                label: token.to_string(),
                kind: OpKind::PushInt(n),
            };
        }
        match self.dictionary.get(upper) {
            Some(Word::Primitive(p)) => Op {
                label: upper.to_string(),
                kind: OpKind::CallPrim(*p),
            },
            _ => Op {
                label: upper.to_string(),
                kind: OpKind::CallByName(upper.to_string()),
            },
        }
    }

    fn execute_ops(&mut self, name: &str, ops: Vec<Op>) {
        self.output.push(format!("call {}", name));
        for op in &ops {
            self.execute_op(op);
            self.emit_trace(&op.label);
        }
    }

    fn execute_op(&mut self, op: &Op) {
        match &op.kind {
            OpKind::PushInt(n) => {
                self.stack.push(Value::Int(*n));
                self.output.push(format!("push {}", n));
            }
            OpKind::CallPrim(p) => self.execute_primitive(*p),
            OpKind::CallByName(name) => match self.dictionary.get(name).cloned() {
                Some(Word::Primitive(prim)) => self.execute_primitive(prim),
                Some(Word::User(inner)) => self.execute_ops(name, inner),
                None => self.output.push(format!("unknown token: {}", name)),
            },
        }
    }

    fn execute_primitive(&mut self, prim: PrimitiveId) {
        match prim {
            PrimitiveId::Dup => self.prim_dup(),
            PrimitiveId::Add => self.prim_add(),
            PrimitiveId::Mul => self.prim_mul(),
            PrimitiveId::Dot => self.prim_dot(),
            PrimitiveId::DotS => self.prim_dot_s(),
            PrimitiveId::Clear => self.prim_clear(),
        }
    }

    fn prim_dup(&mut self) {
        if let Some(top) = self.stack.last().cloned() {
            self.stack.push(top);
            self.output.push("dup".to_string());
        } else {
            self.output.push("dup: stack empty".to_string());
        }
    }

    fn prim_add(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => {
                let r = a + b;
                self.stack.push(Value::Int(r));
                self.output.push(format!("{} {} + -> {}", a, b, r));
            }
            _ => self.output.push("+: need two ints".to_string()),
        }
    }

    fn prim_mul(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => {
                let r = a * b;
                self.stack.push(Value::Int(r));
                self.output.push(format!("{} {} * -> {}", a, b, r));
            }
            _ => self.output.push("*: need two ints".to_string()),
        }
    }

    fn prim_dot(&mut self) {
        match self.pop_int() {
            Some(n) => self.output.push(n.to_string()),
            None => self.output.push(".: stack empty".to_string()),
        }
    }

    fn prim_dot_s(&mut self) {
        let rendered = self
            .stack
            .iter()
            .map(|v| match v {
                Value::Int(n) => n.to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ");
        self.output.push(format!("STACK: {}", rendered));
    }

    fn prim_clear(&mut self) {
        self.stack.clear();
        self.output.push("stack cleared".to_string());
    }

    fn pop_int(&mut self) -> Option<i32> {
        match self.stack.pop() {
            Some(Value::Int(n)) => Some(n),
            None => None,
        }
    }
}
