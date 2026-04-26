use std::collections::{HashMap, HashSet};
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
    Sub,
    Mul,
    Div,
    Mod,
    SlashMod,
    StarSlashMod,
    Eq,
    Lt,
    Gt,
    And,
    Or,
    Xor,
    Invert,
    LShift,
    RShift,
    CompileComma,
    Dot,
    DotS,
    Clear,
    ToR,
    FromR,
    RFetch,
    Fetch,
    Store,
    PlusStore,
    Words,
    Cr,
    Emit,
    Space,
    I,
    J,
    Allot,
    Execute,
    Here,
    Comma,
    Latest,
    Immediate,
    Create,
    If,
    Else,
    Then,
    Begin,
    Until,
    While,
    Repeat,
    Do,
    LoopPrim,
    PlusLoop,
    LeavePrim,
    LBracket,
    RBracket,
    Literal,
    DoesArrow,
    Postpone,
}

#[derive(Clone, Debug)]
enum Word {
    Primitive(PrimitiveId),
    User(Vec<Op>),
    Variable(i32),
    Constant(i32),
    Created {
        data_addr: i32,
        does_ops: Option<Vec<Op>>,
    },
}

#[derive(Clone, Debug)]
enum NextTokenConsumer {
    See,
    Variable,
    Constant(i32),
    Tick,
    Create(i32),
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
    Noop,
    LoopEnter,
    LoopNext(usize),
    LoopNextStep(usize),
    LeaveLoop(usize),
    Does,
    PostponeCall(String),
    PrintStr(String),
}

#[derive(Clone, Debug)]
struct Pending {
    name: Option<String>,
    body: Vec<Op>,
    cf_stack: Vec<usize>,
    leave_stack: Vec<Vec<usize>>,
    pending_postpone: bool,
    anonymous: bool,
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
    return_stack: Vec<Value>,
    memory: Vec<Value>,
    xt_table: Vec<String>,
    output: Vec<String>,
    output_line: String,
    history: Vec<String>,
    trace: Vec<String>,
    dictionary: HashMap<String, Word>,
    immediate_words: HashSet<String>,
    latest: Option<String>,
    compiling: Option<Pending>,
    paused_compile: Option<Pending>,
    next_consumer: Option<NextTokenConsumer>,
    pending_does: Option<Vec<Op>>,
    anon_counter: u32,
}

#[wasm_bindgen]
impl Machine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Machine {
        let mut m = Machine {
            stack: Vec::new(),
            return_stack: Vec::new(),
            memory: Vec::new(),
            xt_table: Vec::new(),
            output: vec![
                "Machine created.".to_string(),
                "Primitives: DUP SWAP DROP OVER ROT + - * / MOD /MOD */MOD = < > AND OR XOR INVERT LSHIFT RSHIFT . .S CLEAR >R R> R@ @ ! +! WORDS CR EMIT SPACE I J ALLOT EXECUTE HERE , COMPILE, LATEST IMMEDIATE CREATE BASE".to_string(),
                "Compile: : ; :NONAME IF ELSE THEN BEGIN UNTIL WHILE REPEAT DO LOOP +LOOP LEAVE [ ] LITERAL DOES> POSTPONE .\"".to_string(),
                "Interactive: SEE <word> | VARIABLE <name> | <val> CONSTANT <name> | ' <word>".to_string(),
            ],
            output_line: String::new(),
            history: Vec::new(),
            trace: Vec::new(),
            dictionary: HashMap::new(),
            immediate_words: HashSet::new(),
            latest: None,
            compiling: None,
            paused_compile: None,
            next_consumer: None,
            pending_does: None,
            anon_counter: 0,
        };
        m.install_primitives();
        m
    }

    pub fn reset(&mut self) {
        self.stack.clear();
        self.return_stack.clear();
        self.compiling = None;
        self.paused_compile = None;
        self.next_consumer = None;
        self.pending_does = None;
        self.output_line.clear();
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
        self.run_tokens(line);
        self.flush_output_line();
    }

    pub fn load_source(&mut self, src: &str) {
        let src = src.trim();
        if src.is_empty() {
            return;
        }

        self.history.push("> [load source]".to_string());
        self.run_tokens(src);
        self.flush_output_line();
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
        let mut text = self.output.join("\n");
        if !self.output_line.is_empty() {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(&self.output_line);
        }
        text
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

    pub fn get_memory_text(&self) -> String {
        self.memory
            .iter()
            .enumerate()
            .map(|(i, v)| match v {
                Value::Int(n) => format!("[{}] {}", i, n),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Machine {
    fn run_tokens(&mut self, src: &str) {
        let chars: Vec<char> = src.chars().collect();
        let mut i = 0;
        let mut comment_depth: u32 = 0;
        while i < chars.len() {
            // Skip whitespace
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            if i >= chars.len() {
                break;
            }
            // Extract one whitespace-delimited token
            let start = i;
            while i < chars.len() && !chars[i].is_whitespace() {
                i += 1;
            }
            let token: String = chars[start..i].iter().collect();

            // Line comment: skip to end of line
            if token == "\\" {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            if token == "(" {
                comment_depth += 1;
                continue;
            }
            if token == ")" {
                if comment_depth > 0 {
                    comment_depth -= 1;
                } else {
                    self.output.push("): unmatched".to_string());
                }
                continue;
            }
            if comment_depth > 0 {
                continue;
            }
            // .": scan until closing " (whitespace-tolerant)
            if token == ".\"" {
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                let mut s = String::new();
                while i < chars.len() && chars[i] != '"' {
                    s.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() {
                    i += 1; // skip closing "
                } else {
                    self.output
                        .push(".\": missing closing quote".to_string());
                }
                self.handle_print_string(s);
                continue;
            }
            self.dispatch_token(&token);
        }
        if comment_depth > 0 {
            self.output
                .push(format!("(: unclosed ({} open)", comment_depth));
        }
    }

    fn handle_print_string(&mut self, s: String) {
        if self.compiling.is_some() {
            let label = format!(".\" {}\"", s);
            let p = self.compiling.as_mut().unwrap();
            p.body.push(Op {
                label,
                kind: OpKind::PrintStr(s),
            });
        } else {
            self.output_line.push_str(&s);
        }
    }

    fn current_base(&self) -> u32 {
        match self.memory.first() {
            Some(Value::Int(b)) if *b >= 2 && *b <= 36 => *b as u32,
            _ => 10,
        }
    }

    fn format_int(n: i32, base: u32) -> String {
        if !(2..=36).contains(&base) {
            return n.to_string();
        }
        let mag: u32 = if n < 0 {
            (n as i64).unsigned_abs() as u32
        } else {
            n as u32
        };
        if mag == 0 {
            return "0".to_string();
        }
        let mut digits = String::new();
        let mut x = mag;
        while x > 0 {
            let d = (x % base) as u8;
            digits.push(if d < 10 {
                (b'0' + d) as char
            } else {
                (b'A' + d - 10) as char
            });
            x /= base;
        }
        let body: String = digits.chars().rev().collect();
        if n < 0 {
            format!("-{}", body)
        } else {
            body
        }
    }

    fn parse_literal(&self, token: &str) -> Option<i32> {
        i32::from_str_radix(token, self.current_base()).ok()
    }

    fn install_primitives(&mut self) {
        let entries: &[(&str, PrimitiveId)] = &[
            ("DUP", PrimitiveId::Dup),
            ("SWAP", PrimitiveId::Swap),
            ("DROP", PrimitiveId::Drop),
            ("OVER", PrimitiveId::Over),
            ("ROT", PrimitiveId::Rot),
            ("+", PrimitiveId::Add),
            ("-", PrimitiveId::Sub),
            ("*", PrimitiveId::Mul),
            ("/", PrimitiveId::Div),
            ("MOD", PrimitiveId::Mod),
            ("/MOD", PrimitiveId::SlashMod),
            ("*/MOD", PrimitiveId::StarSlashMod),
            ("=", PrimitiveId::Eq),
            ("<", PrimitiveId::Lt),
            (">", PrimitiveId::Gt),
            ("AND", PrimitiveId::And),
            ("OR", PrimitiveId::Or),
            ("XOR", PrimitiveId::Xor),
            ("INVERT", PrimitiveId::Invert),
            ("LSHIFT", PrimitiveId::LShift),
            ("RSHIFT", PrimitiveId::RShift),
            ("COMPILE,", PrimitiveId::CompileComma),
            ("IF", PrimitiveId::If),
            ("ELSE", PrimitiveId::Else),
            ("THEN", PrimitiveId::Then),
            ("BEGIN", PrimitiveId::Begin),
            ("UNTIL", PrimitiveId::Until),
            ("WHILE", PrimitiveId::While),
            ("REPEAT", PrimitiveId::Repeat),
            ("DO", PrimitiveId::Do),
            ("LOOP", PrimitiveId::LoopPrim),
            ("+LOOP", PrimitiveId::PlusLoop),
            ("LEAVE", PrimitiveId::LeavePrim),
            ("[", PrimitiveId::LBracket),
            ("]", PrimitiveId::RBracket),
            ("LITERAL", PrimitiveId::Literal),
            ("DOES>", PrimitiveId::DoesArrow),
            ("POSTPONE", PrimitiveId::Postpone),
            (".", PrimitiveId::Dot),
            (".S", PrimitiveId::DotS),
            ("CLEAR", PrimitiveId::Clear),
            (">R", PrimitiveId::ToR),
            ("R>", PrimitiveId::FromR),
            ("R@", PrimitiveId::RFetch),
            ("@", PrimitiveId::Fetch),
            ("!", PrimitiveId::Store),
            ("+!", PrimitiveId::PlusStore),
            ("WORDS", PrimitiveId::Words),
            ("CR", PrimitiveId::Cr),
            ("EMIT", PrimitiveId::Emit),
            ("SPACE", PrimitiveId::Space),
            ("I", PrimitiveId::I),
            ("J", PrimitiveId::J),
            ("ALLOT", PrimitiveId::Allot),
            ("EXECUTE", PrimitiveId::Execute),
            ("HERE", PrimitiveId::Here),
            (",", PrimitiveId::Comma),
            ("LATEST", PrimitiveId::Latest),
            ("IMMEDIATE", PrimitiveId::Immediate),
            ("CREATE", PrimitiveId::Create),
        ];
        for (name, id) in entries {
            self.define_word((*name).to_string(), Word::Primitive(*id));
        }
        // Compile-mode helpers — IMMEDIATE in standard Forth, except `]`
        // which is a normal word that resumes compilation.
        for name in [
            "IF", "ELSE", "THEN", "BEGIN", "UNTIL", "WHILE", "REPEAT", "DO", "LOOP",
            "+LOOP", "LEAVE", "[", "LITERAL", "DOES>", "POSTPONE",
        ] {
            self.immediate_words.insert(name.to_string());
        }
        // BASE: kernel-resident variable at memory[0] (default radix 10).
        let base_addr = self.memory.len() as i32;
        self.memory.push(Value::Int(10));
        self.define_word("BASE".to_string(), Word::Variable(base_addr));
    }

    fn define_word(&mut self, name: String, word: Word) {
        self.dictionary.insert(name.clone(), word);
        self.latest = Some(name);
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
        if let Some(consumer) = self.next_consumer.take() {
            self.handle_consumer(consumer, token);
            return;
        }

        let upper = token.to_ascii_uppercase();

        // ' (tick) works in both interpret and compile modes; the deferred
        // handle_consumer call dispatches based on the current mode.
        if upper == "'" {
            self.next_consumer = Some(NextTokenConsumer::Tick);
            return;
        }

        if self.compiling.is_some() {
            self.dispatch_compile(token, &upper);
            return;
        }

        if upper == "SEE" {
            self.next_consumer = Some(NextTokenConsumer::See);
            return;
        }

        if upper == "VARIABLE" {
            self.next_consumer = Some(NextTokenConsumer::Variable);
            return;
        }

        if upper == "CONSTANT" {
            match self.pop_int() {
                Some(v) => self.next_consumer = Some(NextTokenConsumer::Constant(v)),
                None => self.output.push("CONSTANT: stack empty".to_string()),
            }
            return;
        }

        if upper == ":" {
            self.compiling = Some(Pending {
                name: None,
                body: Vec::new(),
                cf_stack: Vec::new(),
                leave_stack: Vec::new(),
                pending_postpone: false,
                anonymous: false,
            });
            return;
        }

        if upper == ":NONAME" {
            let n = self.anon_counter;
            self.anon_counter += 1;
            let name = format!("<anon-{}>", n);
            self.compiling = Some(Pending {
                name: Some(name),
                body: Vec::new(),
                cf_stack: Vec::new(),
                leave_stack: Vec::new(),
                pending_postpone: false,
                anonymous: true,
            });
            return;
        }
        if upper == ";" {
            self.output.push(";: not compiling".to_string());
            return;
        }

        if let Some(n) = self.parse_literal(token) {
            self.stack.push(Value::Int(n));
            return;
        }

        match self.dictionary.get(&upper).cloned() {
            Some(Word::Primitive(prim)) => self.execute_primitive(prim),
            Some(Word::User(ops)) => self.run_user(&upper, ops),
            Some(Word::Variable(addr)) => {
                self.stack.push(Value::Int(addr));
            }
            Some(Word::Constant(v)) => {
                self.stack.push(Value::Int(v));
            }
            Some(Word::Created { data_addr, does_ops }) => {
                self.stack.push(Value::Int(data_addr));
                if let Some(ops) = does_ops {
                    self.run_user(&upper, ops);
                }
            }
            None => self.output.push(format!("unknown token: {}", token)),
        }
    }

    fn handle_consumer(&mut self, consumer: NextTokenConsumer, token: &str) {
        match consumer {
            NextTokenConsumer::See => self.handle_see(token),
            NextTokenConsumer::Variable => {
                let name = token.to_ascii_uppercase();
                let addr = self.memory.len() as i32;
                self.memory.push(Value::Int(0));
                self.define_word(name.clone(), Word::Variable(addr));
                self.output
                    .push(format!("VARIABLE {} at addr {}", name, addr));
            }
            NextTokenConsumer::Constant(v) => {
                let name = token.to_ascii_uppercase();
                self.define_word(name.clone(), Word::Constant(v));
                self.output.push(format!("CONSTANT {} = {}", name, v));
            }
            NextTokenConsumer::Create(data_addr) => {
                let name = token.to_ascii_uppercase();
                let does_ops = self.pending_does.take();
                let has_does = does_ops.is_some();
                self.define_word(
                    name.clone(),
                    Word::Created { data_addr, does_ops },
                );
                if has_does {
                    self.output.push(format!(
                        "CREATE {} at addr {} (with DOES>)",
                        name, data_addr
                    ));
                } else {
                    self.output
                        .push(format!("CREATE {} at addr {}", name, data_addr));
                }
            }
            NextTokenConsumer::Tick => {
                let upper = token.to_ascii_uppercase();
                if !self.dictionary.contains_key(&upper) {
                    self.output.push(format!("': unknown word {}", token));
                    return;
                }
                let xt = self.intern_xt(&upper);
                if self.compiling.is_some() {
                    let p = self.compiling.as_mut().unwrap();
                    p.body.push(Op {
                        label: format!("' {}", upper),
                        kind: OpKind::PushInt(xt),
                    });
                } else {
                    self.stack.push(Value::Int(xt));
                }
            }
        }
    }

    fn intern_xt(&mut self, name: &str) -> i32 {
        if let Some(idx) = self.xt_table.iter().position(|n| n == name) {
            return idx as i32;
        }
        let idx = self.xt_table.len() as i32;
        self.xt_table.push(name.to_string());
        idx
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
            let name = upper.to_string();
            pending.name = Some(name.clone());
            // Make LATEST point at the in-progress definition so RECURSE-style
            // helpers can resolve it via xt_table while the body is still
            // being compiled.
            self.latest = Some(name);
            return;
        }

        // POSTPONE consumes the next token as its target.
        if pending.pending_postpone {
            let name = upper.to_string();
            if !self.dictionary.contains_key(&name) {
                self.output
                    .push(format!("POSTPONE: unknown word {}", token));
                self.compiling.as_mut().unwrap().pending_postpone = false;
                return;
            }
            let p = self.compiling.as_mut().unwrap();
            p.pending_postpone = false;
            p.body.push(Op {
                label: format!("POSTPONE {}", name),
                kind: OpKind::PostponeCall(name),
            });
            return;
        }

        if upper == ";" {
            let done = self.compiling.take().unwrap();
            if !done.cf_stack.is_empty() {
                let name = done.name.clone().unwrap_or_else(|| "<unnamed>".to_string());
                self.output.push(format!(
                    "compile: {} dropped; {} unclosed IF/ELSE/BEGIN",
                    name,
                    done.cf_stack.len()
                ));
                return;
            }
            let is_anon = done.anonymous;
            let name = done.name.unwrap();
            let body_len = done.body.len();
            self.define_word(name.clone(), Word::User(done.body));
            if is_anon {
                let xt = self.intern_xt(&name);
                self.stack.push(Value::Int(xt));
                self.output
                    .push(format!(":NONAME -> xt {} ({} ops)", xt, body_len));
            } else {
                self.output
                    .push(format!("defined {} ({} tokens)", name, body_len));
            }
            return;
        }

        if upper == ":" {
            self.output
                .push("compile: nested : not allowed".to_string());
            return;
        }

        // Built-in compile helpers (IF/ELSE/THEN, BEGIN/UNTIL/WHILE/REPEAT,
        // DO/LOOP/+LOOP/LEAVE, [, DOES>, POSTPONE, LITERAL) are dict-resident
        // primitives marked IMMEDIATE; they fire via the immediate-words
        // mechanism below, exactly the same path user-defined IMMEDIATE
        // words take. The hardcoded special-case checks that used to live
        // here are gone.
        //
        // Immediate words execute inline at compile time instead of being
        // compiled into the body — the Forth way to extend the compiler.
        if self.immediate_words.contains(upper) {
            if let Some(word) = self.dictionary.get(upper).cloned() {
                match word {
                    Word::Primitive(p) => self.execute_primitive(p),
                    Word::User(ops) => self.run_user(upper, ops),
                    Word::Variable(addr) => self.stack.push(Value::Int(addr)),
                    Word::Constant(v) => self.stack.push(Value::Int(v)),
                    Word::Created { data_addr, does_ops } => {
                        self.stack.push(Value::Int(data_addr));
                        if let Some(ops) = does_ops {
                            self.run_user(upper, ops);
                        }
                    }
                }
                return;
            }
        }

        let op = self.compile_token(token, upper);
        self.compiling.as_mut().unwrap().body.push(op);
    }

    fn handle_see(&mut self, token: &str) {
        let upper = token.to_ascii_uppercase();
        let immediate_tag = if self.immediate_words.contains(&upper) {
            " IMMEDIATE"
        } else {
            ""
        };
        match self.dictionary.get(&upper).cloned() {
            Some(Word::Primitive(p)) => {
                self.output
                    .push(format!("SEE {}: primitive {:?}{}", upper, p, immediate_tag));
            }
            Some(Word::User(ops)) => {
                let body = ops
                    .iter()
                    .map(|op| op.label.clone())
                    .collect::<Vec<_>>()
                    .join(" ");
                self.output
                    .push(format!(": {} {} ;{}", upper, body, immediate_tag));
            }
            Some(Word::Variable(addr)) => {
                let cell = self
                    .addr_to_index(addr)
                    .map(|i| match &self.memory[i] {
                        Value::Int(n) => n.to_string(),
                    })
                    .unwrap_or_else(|| "?".to_string());
                self.output.push(format!(
                    "SEE {}: variable @ addr {} = {}{}",
                    upper, addr, cell, immediate_tag
                ));
            }
            Some(Word::Constant(v)) => {
                self.output
                    .push(format!("SEE {}: constant = {}{}", upper, v, immediate_tag));
            }
            Some(Word::Created { data_addr, does_ops }) => match does_ops {
                Some(ops) => {
                    let body = ops
                        .iter()
                        .map(|op| op.label.clone())
                        .collect::<Vec<_>>()
                        .join(" ");
                    self.output.push(format!(
                        "SEE {}: created @ addr {}, does [ {} ]{}",
                        upper, data_addr, body, immediate_tag
                    ));
                }
                None => self.output.push(format!(
                    "SEE {}: created @ addr {}{}",
                    upper, data_addr, immediate_tag
                )),
            },
            None => {
                self.output.push(format!("SEE: unknown word {}", token));
            }
        }
    }

    fn close_do_loop(&mut self, label: &str, plus: bool) {
        let idx_do = match self.compiling.as_mut().unwrap().cf_stack.pop() {
            Some(i) => i,
            None => {
                self.output
                    .push(format!("compile: {} without DO", label));
                return;
            }
        };
        let leaves = self
            .compiling
            .as_mut()
            .unwrap()
            .leave_stack
            .pop()
            .unwrap_or_default();
        let p = self.compiling.as_mut().unwrap();
        let target = idx_do + 1;
        let kind = if plus {
            OpKind::LoopNextStep(target)
        } else {
            OpKind::LoopNext(target)
        };
        p.body.push(Op {
            label: label.to_string(),
            kind,
        });
        let after = p.body.len();
        for leave_idx in leaves {
            if let OpKind::LeaveLoop(t) = &mut p.body[leave_idx].kind {
                *t = after;
            }
        }
    }

    fn compile_token(&self, token: &str, upper: &str) -> Op {
        if let Some(n) = self.parse_literal(token) {
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

    fn run_user(&mut self, _name: &str, ops: Vec<Op>) {
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
                    frames.push(Frame {
                        ops: inner,
                        pc: 0,
                        return_label: Some(op.label.clone()),
                    });
                }
                Some(Word::Variable(addr)) => {
                    self.stack.push(Value::Int(addr));
                    self.emit_trace(&op.label);
                }
                Some(Word::Constant(v)) => {
                    self.stack.push(Value::Int(v));
                    self.emit_trace(&op.label);
                }
                Some(Word::Created { data_addr, does_ops }) => {
                    self.stack.push(Value::Int(data_addr));
                    self.emit_trace(&op.label);
                    if let Some(inner) = does_ops {
                        frames.push(Frame {
                            ops: inner,
                            pc: 0,
                            return_label: None,
                        });
                    }
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
            OpKind::Noop => {
                self.emit_trace(&op.label);
            }
            OpKind::LoopEnter => {
                let start = self.pop_int();
                let limit = self.pop_int();
                match (limit, start) {
                    (Some(l), Some(s)) => {
                        self.return_stack.push(Value::Int(l));
                        self.return_stack.push(Value::Int(s));
                    }
                    _ => self.output.push("DO: need limit and start".to_string()),
                }
                self.emit_trace(&op.label);
            }
            OpKind::LoopNext(target) => {
                let len = self.return_stack.len();
                if len < 2 {
                    self.output.push("LOOP: return stack underflow".to_string());
                    self.emit_trace(&op.label);
                    return;
                }
                let limit = match &self.return_stack[len - 2] {
                    Value::Int(n) => *n,
                };
                let new_index = match &mut self.return_stack[len - 1] {
                    Value::Int(n) => {
                        *n += 1;
                        *n
                    }
                };
                if new_index < limit {
                    frames.last_mut().unwrap().pc = *target;
                } else {
                    self.return_stack.pop();
                    self.return_stack.pop();
                }
                self.emit_trace(&op.label);
            }
            OpKind::LoopNextStep(target) => {
                let step = match self.pop_int() {
                    Some(n) => n,
                    None => {
                        self.output.push("+LOOP: stack empty".to_string());
                        self.emit_trace(&op.label);
                        return;
                    }
                };
                let len = self.return_stack.len();
                if len < 2 {
                    self.output.push("+LOOP: return stack underflow".to_string());
                    self.emit_trace(&op.label);
                    return;
                }
                let limit = match &self.return_stack[len - 2] {
                    Value::Int(n) => *n,
                };
                let new_index = match &mut self.return_stack[len - 1] {
                    Value::Int(n) => {
                        *n += step;
                        *n
                    }
                };
                let exit = if step >= 0 {
                    new_index >= limit
                } else {
                    new_index < limit
                };
                if exit {
                    self.return_stack.pop();
                    self.return_stack.pop();
                } else {
                    frames.last_mut().unwrap().pc = *target;
                }
                self.emit_trace(&op.label);
            }
            OpKind::LeaveLoop(target) => {
                if self.return_stack.len() >= 2 {
                    self.return_stack.pop();
                    self.return_stack.pop();
                }
                frames.last_mut().unwrap().pc = *target;
                self.emit_trace(&op.label);
            }
            OpKind::Does => {
                let remaining: Vec<Op> = {
                    let frame = frames.last_mut().unwrap();
                    let r = frame.ops[frame.pc..].to_vec();
                    frame.pc = frame.ops.len();
                    r
                };
                self.pending_does = Some(remaining);
                self.emit_trace(&op.label);
            }
            OpKind::PostponeCall(name) => {
                // For non-immediate NAME, splice a CallByName(NAME) into the
                // currently-compiling word so the outer word, at run time,
                // calls NAME. For immediate NAME (the standard Forth case),
                // EXECUTE NAME right now — that's what would happen if NAME
                // had appeared in the outer source where the POSTPONE fires.
                if self.compiling.is_none() {
                    self.output
                        .push(format!("POSTPONE: not compiling, cannot postpone {}", name));
                    self.emit_trace(&op.label);
                    return;
                }
                if self.immediate_words.contains(name) {
                    let word = self.dictionary.get(name).cloned();
                    if let Some(word) = word {
                        match word {
                            Word::Primitive(p) => self.execute_primitive(p),
                            Word::User(ops) => self.run_user(name, ops),
                            Word::Variable(addr) => self.stack.push(Value::Int(addr)),
                            Word::Constant(v) => self.stack.push(Value::Int(v)),
                            Word::Created { data_addr, does_ops } => {
                                self.stack.push(Value::Int(data_addr));
                                if let Some(inner) = does_ops {
                                    self.run_user(name, inner);
                                }
                            }
                        }
                    } else {
                        self.output
                            .push(format!("POSTPONE: word {} no longer defined", name));
                    }
                } else {
                    let p = self.compiling.as_mut().unwrap();
                    p.body.push(Op {
                        label: name.clone(),
                        kind: OpKind::CallByName(name.clone()),
                    });
                }
                self.emit_trace(&op.label);
            }
            OpKind::PrintStr(s) => {
                self.output_line.push_str(s);
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
            PrimitiveId::Sub => self.prim_sub(),
            PrimitiveId::Mul => self.prim_mul(),
            PrimitiveId::Div => self.prim_div(),
            PrimitiveId::Mod => self.prim_mod(),
            PrimitiveId::SlashMod => self.prim_slash_mod(),
            PrimitiveId::StarSlashMod => self.prim_star_slash_mod(),
            PrimitiveId::Eq => self.prim_eq(),
            PrimitiveId::Lt => self.prim_lt(),
            PrimitiveId::Gt => self.prim_gt(),
            PrimitiveId::And => self.prim_and(),
            PrimitiveId::Or => self.prim_or(),
            PrimitiveId::Xor => self.prim_xor(),
            PrimitiveId::Invert => self.prim_invert(),
            PrimitiveId::LShift => self.prim_lshift(),
            PrimitiveId::RShift => self.prim_rshift(),
            PrimitiveId::CompileComma => self.prim_compile_comma(),
            PrimitiveId::If => self.prim_if(),
            PrimitiveId::Else => self.prim_else(),
            PrimitiveId::Then => self.prim_then(),
            PrimitiveId::Begin => self.prim_begin(),
            PrimitiveId::Until => self.prim_until(),
            PrimitiveId::While => self.prim_while(),
            PrimitiveId::Repeat => self.prim_repeat(),
            PrimitiveId::Do => self.prim_do(),
            PrimitiveId::LoopPrim => self.prim_loop(),
            PrimitiveId::PlusLoop => self.prim_plus_loop(),
            PrimitiveId::LeavePrim => self.prim_leave(),
            PrimitiveId::LBracket => self.prim_l_bracket(),
            PrimitiveId::RBracket => self.prim_r_bracket(),
            PrimitiveId::Literal => self.prim_literal(),
            PrimitiveId::DoesArrow => self.prim_does_arrow(),
            PrimitiveId::Postpone => self.prim_postpone(),
            PrimitiveId::Dot => self.prim_dot(),
            PrimitiveId::DotS => self.prim_dot_s(),
            PrimitiveId::Clear => self.prim_clear(),
            PrimitiveId::ToR => self.prim_to_r(),
            PrimitiveId::FromR => self.prim_r_from(),
            PrimitiveId::RFetch => self.prim_r_fetch(),
            PrimitiveId::Fetch => self.prim_fetch(),
            PrimitiveId::Store => self.prim_store(),
            PrimitiveId::PlusStore => self.prim_plus_store(),
            PrimitiveId::Words => self.prim_words(),
            PrimitiveId::Cr => self.prim_cr(),
            PrimitiveId::Emit => self.prim_emit(),
            PrimitiveId::Space => self.prim_space(),
            PrimitiveId::I => self.prim_i(),
            PrimitiveId::J => self.prim_j(),
            PrimitiveId::Allot => self.prim_allot(),
            PrimitiveId::Execute => self.prim_execute(),
            PrimitiveId::Here => self.prim_here(),
            PrimitiveId::Comma => self.prim_comma(),
            PrimitiveId::Latest => self.prim_latest(),
            PrimitiveId::Immediate => self.prim_immediate(),
            PrimitiveId::Create => self.prim_create(),
        }
    }

    fn prim_dup(&mut self) {
        if let Some(top) = self.stack.last().cloned() {
            self.stack.push(top);
        } else {
            self.output.push("dup: stack empty".to_string());
        }
    }

    fn prim_add(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => self.stack.push(Value::Int(a + b)),
            _ => self.output.push("+: need two ints".to_string()),
        }
    }

    fn prim_sub(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => self.stack.push(Value::Int(a - b)),
            _ => self.output.push("-: need two ints".to_string()),
        }
    }

    fn prim_mul(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => self.stack.push(Value::Int(a * b)),
            _ => self.output.push("*: need two ints".to_string()),
        }
    }

    fn prim_div(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => match a.checked_div(b) {
                Some(r) => self.stack.push(Value::Int(r)),
                None => self.output.push("/: divide by zero".to_string()),
            },
            _ => self.output.push("/: need two ints".to_string()),
        }
    }

    fn prim_mod(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => match a.checked_rem(b) {
                Some(r) => self.stack.push(Value::Int(r)),
                None => self.output.push("MOD: divide by zero".to_string()),
            },
            _ => self.output.push("MOD: need two ints".to_string()),
        }
    }

    fn prim_slash_mod(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => {
                let r = a.checked_rem(b);
                let q = a.checked_div(b);
                match (r, q) {
                    (Some(r), Some(q)) => {
                        self.stack.push(Value::Int(r));
                        self.stack.push(Value::Int(q));
                    }
                    _ => self.output.push("/MOD: divide by zero".to_string()),
                }
            }
            _ => self.output.push("/MOD: need two ints".to_string()),
        }
    }

    fn prim_star_slash_mod(&mut self) {
        let c = self.pop_int();
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b, c) {
            (Some(a), Some(b), Some(c)) => {
                let prod = a.wrapping_mul(b);
                let r = prod.checked_rem(c);
                let q = prod.checked_div(c);
                match (r, q) {
                    (Some(r), Some(q)) => {
                        self.stack.push(Value::Int(r));
                        self.stack.push(Value::Int(q));
                    }
                    _ => self.output.push("*/MOD: divide by zero".to_string()),
                }
            }
            _ => self.output.push("*/MOD: need three ints".to_string()),
        }
    }

    fn prim_dot(&mut self) {
        match self.pop_int() {
            Some(n) => {
                let base = self.current_base();
                self.output_line
                    .push_str(&format!("{} ", Self::format_int(n, base)));
            }
            None => self.output.push(".: stack empty".to_string()),
        }
    }

    fn flush_output_line(&mut self) {
        if !self.output_line.is_empty() {
            let line = std::mem::take(&mut self.output_line);
            self.output.push(line);
        }
    }

    fn prim_dot_s(&mut self) {
        let base = self.current_base();
        let rendered = self
            .stack
            .iter()
            .map(|v| match v {
                Value::Int(n) => Self::format_int(*n, base),
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
            }
            _ => self.output.push("swap: need two values".to_string()),
        }
    }

    fn prim_drop(&mut self) {
        if self.stack.pop().is_none() {
            self.output.push("drop: stack empty".to_string());
        }
    }

    fn prim_over(&mut self) {
        if self.stack.len() < 2 {
            self.output.push("over: need two values".to_string());
            return;
        }
        let a = self.stack[self.stack.len() - 2].clone();
        self.stack.push(a);
    }

    fn prim_rot(&mut self) {
        if self.stack.len() < 3 {
            self.output.push("rot: need three values".to_string());
            return;
        }
        let len = self.stack.len();
        let a = self.stack.remove(len - 3);
        self.stack.push(a);
    }

    fn prim_eq(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => {
                self.stack.push(Value::Int(if a == b { -1 } else { 0 }));
            }
            _ => self.output.push("=: need two ints".to_string()),
        }
    }

    fn prim_lt(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => {
                self.stack.push(Value::Int(if a < b { -1 } else { 0 }));
            }
            _ => self.output.push("<: need two ints".to_string()),
        }
    }

    fn prim_to_r(&mut self) {
        match self.stack.pop() {
            Some(v) => self.return_stack.push(v),
            None => self.output.push(">R: stack empty".to_string()),
        }
    }

    fn prim_r_from(&mut self) {
        match self.return_stack.pop() {
            Some(v) => self.stack.push(v),
            None => self.output.push("R>: return stack empty".to_string()),
        }
    }

    fn prim_fetch(&mut self) {
        match self.pop_int() {
            Some(addr) => match self.addr_to_index(addr) {
                Some(idx) => self.stack.push(self.memory[idx].clone()),
                None => self.output.push(format!("@: bad address {}", addr)),
            },
            None => self.output.push("@: stack empty".to_string()),
        }
    }

    fn prim_store(&mut self) {
        let a = self.pop_int();
        let v = self.stack.pop();
        match (v, a) {
            (Some(val), Some(addr)) => match self.addr_to_index(addr) {
                Some(idx) => self.memory[idx] = val,
                None => self.output.push(format!("!: bad address {}", addr)),
            },
            _ => self.output.push("!: need value and addr".to_string()),
        }
    }

    fn prim_plus_store(&mut self) {
        let a = self.pop_int();
        let d = self.pop_int();
        match (d, a) {
            (Some(delta), Some(addr)) => match self.addr_to_index(addr) {
                Some(idx) => match &mut self.memory[idx] {
                    Value::Int(n) => *n += delta,
                },
                None => self.output.push(format!("+!: bad address {}", addr)),
            },
            _ => self.output.push("+!: need delta and addr".to_string()),
        }
    }

    fn prim_words(&mut self) {
        let mut names: Vec<_> = self.dictionary.keys().cloned().collect();
        names.sort();
        self.output.push(format!("WORDS: {}", names.join(" ")));
    }

    fn prim_cr(&mut self) {
        self.flush_output_line_force();
    }

    fn flush_output_line_force(&mut self) {
        let line = std::mem::take(&mut self.output_line);
        self.output.push(line);
    }

    fn prim_emit(&mut self) {
        match self.pop_int() {
            Some(n) => {
                let c = std::char::from_u32(n as u32).unwrap_or('?');
                self.output_line.push(c);
            }
            None => self.output.push("EMIT: stack empty".to_string()),
        }
    }

    fn prim_space(&mut self) {
        self.output_line.push(' ');
    }

    fn prim_i(&mut self) {
        match self.return_stack.last().cloned() {
            Some(v) => self.stack.push(v),
            None => self.output.push("I: return stack empty".to_string()),
        }
    }

    fn prim_j(&mut self) {
        let len = self.return_stack.len();
        if len < 3 {
            self.output.push("J: return stack too shallow".to_string());
            return;
        }
        self.stack.push(self.return_stack[len - 3].clone());
    }

    fn prim_here(&mut self) {
        self.stack.push(Value::Int(self.memory.len() as i32));
    }

    fn prim_comma(&mut self) {
        match self.pop_int() {
            Some(n) => self.memory.push(Value::Int(n)),
            None => self.output.push(",: stack empty".to_string()),
        }
    }

    fn prim_latest(&mut self) {
        let name = match &self.latest {
            Some(n) => n.clone(),
            None => {
                self.output
                    .push("LATEST: no words defined".to_string());
                return;
            }
        };
        let xt = self.intern_xt(&name);
        self.stack.push(Value::Int(xt));
    }

    fn prim_immediate(&mut self) {
        match &self.latest {
            Some(name) => {
                let name = name.clone();
                self.immediate_words.insert(name);
            }
            None => self
                .output
                .push("IMMEDIATE: no words defined".to_string()),
        }
    }

    fn prim_create(&mut self) {
        let addr = self.memory.len() as i32;
        self.next_consumer = Some(NextTokenConsumer::Create(addr));
    }

    fn prim_execute(&mut self) {
        let xt = match self.pop_int() {
            Some(n) => n,
            None => {
                self.output.push("EXECUTE: stack empty".to_string());
                return;
            }
        };
        let idx = xt as usize;
        if xt < 0 || idx >= self.xt_table.len() {
            self.output.push(format!("EXECUTE: bad xt {}", xt));
            return;
        }
        let name = self.xt_table[idx].clone();
        match self.dictionary.get(&name).cloned() {
            Some(Word::Primitive(p)) => self.execute_primitive(p),
            Some(Word::User(ops)) => self.run_user(&name, ops),
            Some(Word::Variable(addr)) => self.stack.push(Value::Int(addr)),
            Some(Word::Constant(v)) => self.stack.push(Value::Int(v)),
            Some(Word::Created { data_addr, does_ops }) => {
                self.stack.push(Value::Int(data_addr));
                if let Some(ops) = does_ops {
                    self.run_user(&name, ops);
                }
            }
            None => self
                .output
                .push(format!("EXECUTE: word {} no longer defined", name)),
        }
    }

    fn prim_allot(&mut self) {
        match self.pop_int() {
            Some(n) if n >= 0 => {
                for _ in 0..n {
                    self.memory.push(Value::Int(0));
                }
                self.output.push(format!(
                    "ALLOT {} cells (memory now {})",
                    n,
                    self.memory.len()
                ));
            }
            Some(n) => self
                .output
                .push(format!("ALLOT: negative count {}", n)),
            None => self.output.push("ALLOT: stack empty".to_string()),
        }
    }

    fn addr_to_index(&self, addr: i32) -> Option<usize> {
        if addr < 0 {
            return None;
        }
        let idx = addr as usize;
        if idx >= self.memory.len() {
            return None;
        }
        Some(idx)
    }

    fn prim_r_fetch(&mut self) {
        match self.return_stack.last().cloned() {
            Some(v) => self.stack.push(v),
            None => self.output.push("R@: return stack empty".to_string()),
        }
    }

    fn prim_gt(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => {
                self.stack.push(Value::Int(if a > b { -1 } else { 0 }));
            }
            _ => self.output.push(">: need two ints".to_string()),
        }
    }

    fn prim_and(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => self.stack.push(Value::Int(a & b)),
            _ => self.output.push("AND: need two ints".to_string()),
        }
    }

    fn prim_or(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => self.stack.push(Value::Int(a | b)),
            _ => self.output.push("OR: need two ints".to_string()),
        }
    }

    fn prim_xor(&mut self) {
        let b = self.pop_int();
        let a = self.pop_int();
        match (a, b) {
            (Some(a), Some(b)) => self.stack.push(Value::Int(a ^ b)),
            _ => self.output.push("XOR: need two ints".to_string()),
        }
    }

    fn prim_invert(&mut self) {
        match self.pop_int() {
            Some(n) => self.stack.push(Value::Int(!n)),
            None => self.output.push("INVERT: stack empty".to_string()),
        }
    }

    fn prim_lshift(&mut self) {
        let n = self.pop_int();
        let a = self.pop_int();
        match (a, n) {
            (Some(a), Some(n)) => {
                let r = (a as u32).wrapping_shl(n as u32) as i32;
                self.stack.push(Value::Int(r));
            }
            _ => self.output.push("LSHIFT: need two ints".to_string()),
        }
    }

    fn prim_rshift(&mut self) {
        let n = self.pop_int();
        let a = self.pop_int();
        match (a, n) {
            (Some(a), Some(n)) => {
                let r = (a as u32).wrapping_shr(n as u32) as i32;
                self.stack.push(Value::Int(r));
            }
            _ => self.output.push("RSHIFT: need two ints".to_string()),
        }
    }

    fn prim_if(&mut self) {
        let p = match self.compiling.as_mut() {
            Some(p) => p,
            None => {
                self.output.push("IF: not compiling".to_string());
                return;
            }
        };
        let idx = p.body.len();
        p.body.push(Op {
            label: "IF".to_string(),
            kind: OpKind::Branch0(0),
        });
        p.cf_stack.push(idx);
    }

    fn prim_else(&mut self) {
        let branch_idx = match self.compiling.as_mut() {
            Some(p) => match p.cf_stack.pop() {
                Some(i) => i,
                None => {
                    self.output.push("ELSE: not in IF".to_string());
                    return;
                }
            },
            None => {
                self.output.push("ELSE: not compiling".to_string());
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
    }

    fn prim_then(&mut self) {
        let idx = match self.compiling.as_mut() {
            Some(p) => match p.cf_stack.pop() {
                Some(i) => i,
                None => {
                    self.output.push("THEN: not in IF/ELSE".to_string());
                    return;
                }
            },
            None => {
                self.output.push("THEN: not compiling".to_string());
                return;
            }
        };
        let p = self.compiling.as_mut().unwrap();
        let target = p.body.len();
        p.body.push(Op {
            label: "THEN".to_string(),
            kind: OpKind::Noop,
        });
        match &mut p.body[idx].kind {
            OpKind::Branch0(t) | OpKind::Jump(t) => *t = target,
            _ => {}
        }
    }

    fn prim_begin(&mut self) {
        let p = match self.compiling.as_mut() {
            Some(p) => p,
            None => {
                self.output.push("BEGIN: not compiling".to_string());
                return;
            }
        };
        let idx = p.body.len();
        p.body.push(Op {
            label: "BEGIN".to_string(),
            kind: OpKind::Noop,
        });
        p.cf_stack.push(idx);
    }

    fn prim_until(&mut self) {
        let target = match self.compiling.as_mut() {
            Some(p) => match p.cf_stack.pop() {
                Some(i) => i,
                None => {
                    self.output.push("UNTIL: not in BEGIN".to_string());
                    return;
                }
            },
            None => {
                self.output.push("UNTIL: not compiling".to_string());
                return;
            }
        };
        let p = self.compiling.as_mut().unwrap();
        p.body.push(Op {
            label: "UNTIL".to_string(),
            kind: OpKind::Branch0(target),
        });
    }

    fn prim_while(&mut self) {
        let p = match self.compiling.as_mut() {
            Some(p) => p,
            None => {
                self.output.push("WHILE: not compiling".to_string());
                return;
            }
        };
        let idx = p.body.len();
        p.body.push(Op {
            label: "WHILE".to_string(),
            kind: OpKind::Branch0(0),
        });
        p.cf_stack.push(idx);
    }

    fn prim_repeat(&mut self) {
        let (while_idx, begin_idx) = {
            let p = match self.compiling.as_mut() {
                Some(p) => p,
                None => {
                    self.output.push("REPEAT: not compiling".to_string());
                    return;
                }
            };
            let w = match p.cf_stack.pop() {
                Some(i) => i,
                None => {
                    self.output.push("REPEAT: not in WHILE".to_string());
                    return;
                }
            };
            let b = match p.cf_stack.pop() {
                Some(i) => i,
                None => {
                    self.output.push("REPEAT: not in BEGIN".to_string());
                    p.cf_stack.push(w);
                    return;
                }
            };
            (w, b)
        };
        let p = self.compiling.as_mut().unwrap();
        p.body.push(Op {
            label: "REPEAT".to_string(),
            kind: OpKind::Jump(begin_idx),
        });
        let target = p.body.len();
        if let OpKind::Branch0(t) = &mut p.body[while_idx].kind {
            *t = target;
        }
    }

    fn prim_do(&mut self) {
        let p = match self.compiling.as_mut() {
            Some(p) => p,
            None => {
                self.output.push("DO: not compiling".to_string());
                return;
            }
        };
        let idx = p.body.len();
        p.body.push(Op {
            label: "DO".to_string(),
            kind: OpKind::LoopEnter,
        });
        p.cf_stack.push(idx);
        p.leave_stack.push(Vec::new());
    }

    fn prim_loop(&mut self) {
        if self.compiling.is_none() {
            self.output.push("LOOP: not compiling".to_string());
            return;
        }
        self.close_do_loop("LOOP", false);
    }

    fn prim_plus_loop(&mut self) {
        if self.compiling.is_none() {
            self.output.push("+LOOP: not compiling".to_string());
            return;
        }
        self.close_do_loop("+LOOP", true);
    }

    fn prim_leave(&mut self) {
        let p = match self.compiling.as_mut() {
            Some(p) => p,
            None => {
                self.output.push("LEAVE: not compiling".to_string());
                return;
            }
        };
        if p.leave_stack.is_empty() {
            self.output.push("LEAVE: not in DO".to_string());
            return;
        }
        let idx = p.body.len();
        p.body.push(Op {
            label: "LEAVE".to_string(),
            kind: OpKind::LeaveLoop(0),
        });
        let depth = p.leave_stack.len();
        p.leave_stack[depth - 1].push(idx);
    }

    fn prim_l_bracket(&mut self) {
        self.paused_compile = self.compiling.take();
    }

    fn prim_r_bracket(&mut self) {
        match self.paused_compile.take() {
            Some(p) => self.compiling = Some(p),
            None => self.output.push("]: not paused".to_string()),
        }
    }

    fn prim_literal(&mut self) {
        if self.compiling.is_none() {
            self.output.push("LITERAL: not compiling".to_string());
            return;
        }
        let n = match self.pop_int() {
            Some(n) => n,
            None => {
                self.output.push("LITERAL: stack empty".to_string());
                return;
            }
        };
        let p = self.compiling.as_mut().unwrap();
        p.body.push(Op {
            label: n.to_string(),
            kind: OpKind::PushInt(n),
        });
    }

    fn prim_does_arrow(&mut self) {
        let p = match self.compiling.as_mut() {
            Some(p) => p,
            None => {
                self.output.push("DOES>: not compiling".to_string());
                return;
            }
        };
        p.body.push(Op {
            label: "DOES>".to_string(),
            kind: OpKind::Does,
        });
    }

    fn prim_postpone(&mut self) {
        match self.compiling.as_mut() {
            Some(p) => p.pending_postpone = true,
            None => self
                .output
                .push("POSTPONE: not compiling".to_string()),
        }
    }

    fn prim_compile_comma(&mut self) {
        let xt = match self.pop_int() {
            Some(n) => n,
            None => {
                self.output.push("COMPILE,: stack empty".to_string());
                return;
            }
        };
        let idx = xt as usize;
        if xt < 0 || idx >= self.xt_table.len() {
            self.output.push(format!("COMPILE,: bad xt {}", xt));
            return;
        }
        let name = self.xt_table[idx].clone();
        if let Some(p) = self.compiling.as_mut() {
            p.body.push(Op {
                label: name.clone(),
                kind: OpKind::CallByName(name),
            });
        } else {
            self.output
                .push("COMPILE,: not compiling".to_string());
        }
    }

    fn pop_int(&mut self) -> Option<i32> {
        match self.stack.pop() {
            Some(Value::Int(n)) => Some(n),
            None => None,
        }
    }
}
