use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug)]
enum Value {
    Int(i32),
}

#[derive(Clone, Copy, Debug)]
enum PrimitiveId {
    Dup,
    Swap,
    Drop,
    Over,
    Rot,
    Add,
    Mul,
    Eq,
    Lt,
    Gt,
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
    Branch0(usize),
    Jump(usize),
}

#[derive(Clone, Debug)]
struct Pending {
    name: Option<String>,
    body: Vec<Op>,
    cf_stack: Vec<usize>,
}

#[derive(Clone, Debug)]
struct Frame {
    ops: Vec<Op>,
    pc: usize,
    return_label: Option<String>,
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
                "Primitives: DUP SWAP DROP OVER ROT + * = < > . .S CLEAR".to_string(),
                "Compile: : ; IF ELSE THEN".to_string(),
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
        let entries: &[(&str, PrimitiveId)] = &[
            ("DUP", PrimitiveId::Dup),
            ("SWAP", PrimitiveId::Swap),
            ("DROP", PrimitiveId::Drop),
            ("OVER", PrimitiveId::Over),
            ("ROT", PrimitiveId::Rot),
            ("+", PrimitiveId::Add),
            ("*", PrimitiveId::Mul),
            ("=", PrimitiveId::Eq),
            ("<", PrimitiveId::Lt),
            (">", PrimitiveId::Gt),
            (".", PrimitiveId::Dot),
            (".S", PrimitiveId::DotS),
            ("CLEAR", PrimitiveId::Clear),
        ];
        for (name, id) in entries {
            self.dictionary
                .insert((*name).to_string(), Word::Primitive(*id));
        }
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
                cf_stack: Vec::new(),
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
            Some(Word::User(ops)) => self.run_user(&upper, ops),
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
            if !done.cf_stack.is_empty() {
                let name = done.name.clone().unwrap_or_else(|| "<unnamed>".to_string());
                self.output.push(format!(
                    "compile: {} dropped; {} unclosed IF/ELSE",
                    name,
                    done.cf_stack.len()
                ));
                return;
            }
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

        if upper == "IF" {
            let p = self.compiling.as_mut().unwrap();
            let idx = p.body.len();
            p.body.push(Op {
                label: "IF".to_string(),
                kind: OpKind::Branch0(0),
            });
            p.cf_stack.push(idx);
            return;
        }

        if upper == "ELSE" {
            let branch_idx = match self.compiling.as_mut().unwrap().cf_stack.pop() {
                Some(i) => i,
                None => {
                    self.output.push("compile: ELSE without IF".to_string());
                    return;
                }
            };
            let p = self.compiling.as_mut().unwrap();
            let jump_idx = p.body.len();
            p.body.push(Op {
                label: "ELSE".to_string(),
                kind: OpKind::Jump(0),
            });
            let target = p.body.len();
            if let OpKind::Branch0(t) = &mut p.body[branch_idx].kind {
                *t = target;
            }
            p.cf_stack.push(jump_idx);
            return;
        }

        if upper == "THEN" {
            let idx = match self.compiling.as_mut().unwrap().cf_stack.pop() {
                Some(i) => i,
                None => {
                    self.output.push("compile: THEN without IF".to_string());
                    return;
                }
            };
            let p = self.compiling.as_mut().unwrap();
            let target = p.body.len();
            match &mut p.body[idx].kind {
                OpKind::Branch0(t) | OpKind::Jump(t) => *t = target,
                _ => {}
            }
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

    fn run_user(&mut self, name: &str, ops: Vec<Op>) {
        self.output.push(format!("call {}", name));
        let mut frames: Vec<Frame> = vec![Frame {
            ops,
            pc: 0,
            return_label: None,
        }];

        while !frames.is_empty() {
            let next = {
                let frame = frames.last_mut().unwrap();
                if frame.pc >= frame.ops.len() {
                    None
                } else {
                    let op = frame.ops[frame.pc].clone();
                    frame.pc += 1;
                    Some(op)
                }
            };

            match next {
                None => {
                    let popped = frames.pop().unwrap();
                    if let Some(label) = popped.return_label {
                        self.emit_trace(&label);
                    }
                }
                Some(op) => self.exec_op_vm(&op, &mut frames),
            }
        }
    }

    fn exec_op_vm(&mut self, op: &Op, frames: &mut Vec<Frame>) {
        match &op.kind {
            OpKind::PushInt(n) => {
                self.stack.push(Value::Int(*n));
                self.output.push(format!("push {}", n));
                self.emit_trace(&op.label);
            }
            OpKind::CallPrim(p) => {
                self.execute_primitive(*p);
                self.emit_trace(&op.label);
            }
            OpKind::CallByName(name) => match self.dictionary.get(name).cloned() {
                Some(Word::Primitive(prim)) => {
                    self.execute_primitive(prim);
                    self.emit_trace(&op.label);
                }
                Some(Word::User(inner)) => {
                    self.output.push(format!("call {}", name));
                    frames.push(Frame {
                        ops: inner,
                        pc: 0,
                        return_label: Some(op.label.clone()),
                    });
                }
                None => {
                    self.output.push(format!("unknown token: {}", name));
                    self.emit_trace(&op.label);
                }
            },
            OpKind::Branch0(target) => {
                let flag = self.pop_int();
                if flag == Some(0) {
                    frames.last_mut().unwrap().pc = *target;
                } else if flag.is_none() {
                    self.output.push("IF: stack empty".to_string());
                }
                self.emit_trace(&op.label);
            }
            OpKind::Jump(target) => {
                frames.last_mut().unwrap().pc = *target;
                self.emit_trace(&op.label);
            }
        }
    }

    fn execute_primitive(&mut self, prim: PrimitiveId) {
        match prim {
            PrimitiveId::Dup => self.prim_dup(),
            PrimitiveId::Swap => self.prim_swap(),
            PrimitiveId::Drop => self.prim_drop(),
            PrimitiveId::Over => self.prim_over(),
            PrimitiveId::Rot => self.prim_rot(),
            PrimitiveId::Add => self.prim_add(),
            PrimitiveId::Mul => self.prim_mul(),
            PrimitiveId::Eq => self.prim_eq(),
            PrimitiveId::Lt => self.prim_lt(),
            PrimitiveId::Gt => self.prim_gt(),
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

    fn prim_swap(&mut self) {
        let b = self.stack.pop();
        let a = self.stack.pop();
        match (a, b) {
            (Some(a), Some(b)) => {
                self.stack.push(b);
                self.stack.push(a);
                self.output.push("swap".to_string());
            }
            _ => self.output.push("swap: need two values".to_string()),
        }
    }

    fn prim_drop(&mut self) {
        match self.stack.pop() {
            Some(_) => self.output.push("drop".to_string()),
            None => self.output.push("drop: stack empty".to_string()),
        }
    }

    fn prim_over(&mut self) {
        if self.stack.len() < 2 {
            self.output.push("over: need two values".to_string());
            return;
        }
        let a = self.stack[self.stack.len() - 2].clone();
        self.stack.push(a);
        self.output.push("over".to_string());
    }

    fn prim_rot(&mut self) {
        if self.stack.len() < 3 {
            self.output.push("rot: need three values".to_string());
            return;
        }
        let len = self.stack.len();
        let a = self.stack.remove(len - 3);
        self.stack.push(a);
        self.output.push("rot".to_string());
    }

    fn prim_eq(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => {
                let flag = if a == b { -1 } else { 0 };
                self.stack.push(Value::Int(flag));
                self.output.push(format!("{} {} = -> {}", a, b, flag));
            }
            _ => self.output.push("=: need two ints".to_string()),
        }
    }

    fn prim_lt(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => {
                let flag = if a < b { -1 } else { 0 };
                self.stack.push(Value::Int(flag));
                self.output.push(format!("{} {} < -> {}", a, b, flag));
            }
            _ => self.output.push("<: need two ints".to_string()),
        }
    }

    fn prim_gt(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => {
                let flag = if a > b { -1 } else { 0 };
                self.stack.push(Value::Int(flag));
                self.output.push(format!("{} {} > -> {}", a, b, flag));
            }
            _ => self.output.push(">: need two ints".to_string()),
        }
    }

    fn pop_int(&mut self) -> Option<i32> {
        match self.stack.pop() {
            Some(Value::Int(n)) => Some(n),
            None => None,
        }
    }
}
