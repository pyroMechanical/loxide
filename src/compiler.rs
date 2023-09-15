use crate::{
    chunk::{operations::OpCode, Chunk},
    object::{ObjFunction, Object},
    scanner::{Scanner, Token, TokenKind},
    value::copy_string,
    value::Value,
    vm::{InterpretError, VM},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    fn next(&self) -> Self {
        match self {
            Self::None => Self::Assignment,
            Self::Assignment => Self::Or,
            Self::Or => Self::And,
            Self::And => Self::Equality,
            Self::Equality => Self::Comparison,
            Self::Comparison => Self::Term,
            Self::Term => Self::Factor,
            Self::Factor => Self::Unary,
            Self::Unary => Self::Call,
            Self::Call => Self::Primary,
            Self::Primary => Self::Primary,
        }
    }
}

struct ParseRule<'a, 'b> {
    prefix: Option<&'a dyn Fn(&'a mut Parser<'b>, bool) -> ()>,
    infix: Option<&'a dyn Fn(&'a mut Parser<'b>, bool) -> ()>,
    precedence: Precedence,
}

fn get_rule<'a, 'b>(kind: TokenKind) -> ParseRule<'a, 'b> {
    match kind {
        TokenKind::LeftParen => ParseRule {
            prefix: Some(&Parser::grouping),
            infix: Some(&Parser::call),
            precedence: Precedence::Call,
        },
        TokenKind::Bang => ParseRule {
            prefix: Some(&Parser::unary),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Minus => ParseRule {
            prefix: Some(&Parser::unary),
            infix: Some(&Parser::binary),
            precedence: Precedence::Term,
        },
        TokenKind::Plus => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Term,
        },
        TokenKind::Slash => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Factor,
        },
        TokenKind::Star => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Factor,
        },
        TokenKind::BangEqual => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::EqualEqual => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::Greater => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::GreaterEqual => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::Less => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::LessEqual => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::Number => ParseRule {
            prefix: Some(&Parser::number),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::And => ParseRule {
            prefix: None,
            infix: Some(&Parser::and),
            precedence: Precedence::And,
        },
        TokenKind::Or => ParseRule {
            prefix: None,
            infix: Some(&Parser::or),
            precedence: Precedence::Or,
        },
        TokenKind::True | TokenKind::False | TokenKind::Nil => ParseRule {
            prefix: Some(&Parser::literal),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Identifier => ParseRule {
            prefix: Some(&Parser::variable),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::String => ParseRule {
            prefix: Some(&Parser::string),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Dot => ParseRule {
            prefix: None,
            infix: Some(&Parser::dot),
            precedence: Precedence::Call,
        },
        _ => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    }
}

fn error(token: Token, message: &str, had_error: &mut bool, panic_mode: &mut bool) {
    if *panic_mode {
        return;
    }
    *panic_mode = true;
    *had_error = true;
    eprint!("[line {}] Error", token.line());
    match token.kind() {
        TokenKind::Error => (),
        _ => {
            eprint!(" at '{}': ", token.as_str());
        }
    }

    eprintln!("{}", message);
}
#[derive(Clone, Copy)]
struct Local<'a> {
    name: &'a str,
    depth: Option<i32>,
    is_captured: bool,
}

impl<'a> Local<'a> {
    fn new(name: &'a str, depth: Option<i32>) -> Self {
        Self {
            name,
            depth,
            is_captured: false,
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
enum FunctionType {
    Function,
    Script,
}
#[derive(Clone, Copy)]
struct Upvalue {
    index: u8,
    is_local: bool,
}

pub struct Compiler<'a> {
    enclosing: *mut Compiler<'a>,
    function: *mut ObjFunction,
    function_type: FunctionType,
    locals: [Local<'a>; 256],
    local_count: usize,
    upvalues: [Upvalue; 256],
    scope_depth: i32,
}

impl<'a> Compiler<'a> {
    fn new(vm: &mut VM, name: Option<Token<'a>>) -> Self {
        let function_type = if name.is_none() {
            FunctionType::Script
        } else {
            FunctionType::Function
        };
        let mut compiler = Self {
            enclosing: std::ptr::null_mut(),
            function: std::ptr::null_mut(),
            function_type,
            locals: [Local::new("", None); 256],
            local_count: 1,
            upvalues: [Upvalue {
                index: 0,
                is_local: false,
            }; 256],
            scope_depth: 0,
        };
        let function = Object::new_function(vm, Some(&mut compiler));
        compiler.function = function;
        compiler.locals[0].depth = Some(0);

        let function_name = name.map(|token| {
            let str = token.as_str();
            Object::new_string(str, vm, Some(&mut compiler))
        });
        unsafe {
            (*compiler.function).name = function_name.unwrap_or(std::ptr::null_mut());
        }
        compiler
    }

    fn resolve_local(
        &self,
        name: &str,
        previous: Token,
        had_error: &mut bool,
        panic_mode: &mut bool,
    ) -> Option<u8> {
        for i in (0..self.local_count).rev() {
            let local = &self.locals[i];
            if local.name == name {
                if local.depth.is_none() {
                    error(
                        previous,
                        "Can't read local variable in its own initializer.",
                        had_error,
                        panic_mode,
                    );
                }
                return Some(i as u8);
            }
        }
        return None;
    }

    fn resolve_upvalue(
        &mut self,
        name: &str,
        previous: Token,
        had_error: &mut bool,
        panic_mode: &mut bool,
    ) -> Option<u8> {
        if self.enclosing.is_null() {
            return None;
        }
        let enclosing = unsafe { &mut *self.enclosing };
        let local = enclosing.resolve_local(name, previous, had_error, panic_mode);
        if let Some(local) = local {
            enclosing.locals[local as usize].is_captured = true;
            return self.add_upvalue(local, true, previous, had_error, panic_mode);
        }

        let upvalue = enclosing.resolve_upvalue(name, previous, had_error, panic_mode);
        if let Some(upvalue) = upvalue {
            return self.add_upvalue(upvalue, false, previous, had_error, panic_mode);
        }
        return None;
    }

    fn add_upvalue(
        &mut self,
        index: u8,
        is_local: bool,
        previous: Token,
        had_error: &mut bool,
        panic_mode: &mut bool,
    ) -> Option<u8> {
        println!("upvalue index: {}", index);
        let upvalue_count = unsafe { &*self.function }.upvalue_count;
        for i in 0..upvalue_count {
            let upvalue = self.upvalues[i];
            if upvalue.index == index && upvalue.is_local == is_local {
                return Some(i as u8);
            }
        }
        if upvalue_count == u8::MAX as usize {
            error(
                previous,
                "Too many closure variables in function.",
                had_error,
                panic_mode,
            );
        }

        self.upvalues[upvalue_count].is_local = is_local;
        self.upvalues[upvalue_count].index = index;
        unsafe { &mut *self.function }.upvalue_count += 1;
        return Some(upvalue_count as u8);
    }

    pub fn mark_compiler_roots(&mut self, gray_stack: &mut Vec<*mut Object>) {
        let mut compiler = self as *mut Compiler<'a>;
        while !compiler.is_null() {
            let current = unsafe { &*compiler };
            crate::allocate::mark_object(current.function as *mut Object, gray_stack);
            compiler = current.enclosing;
        }
    }
}

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    previous: Token<'a>,
    current: Token<'a>,
    compiler: Compiler<'a>,
    vm: &'a mut VM,
    panic_mode: bool,
    had_error: bool,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str, vm: &'a mut VM) -> Parser<'a> {
        Parser {
            scanner: Scanner::new(source),
            previous: Token::default(),
            current: Token::default(),
            compiler: Compiler::new(vm, None),
            vm,
            panic_mode: false,
            had_error: false,
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.current.kind() == kind
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) {
        let Self {
            scanner,
            previous,
            current,
            panic_mode,
            had_error,
            ..
        } = self;
        *previous = *current;
        'skip_errors: for token in scanner {
            *current = token;
            if current.kind() != TokenKind::Error {
                break 'skip_errors;
            }
            error(token, token.as_str(), had_error, panic_mode);
        }
    }

    fn consume(&mut self, expected: TokenKind, error_message: &str) {
        if self.current.kind() == expected {
            self.advance();
        } else {
            error(
                self.current,
                error_message,
                &mut self.had_error,
                &mut self.panic_mode,
            )
        }
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut unsafe { &mut *self.compiler.function }.chunk
    }

    fn emit_byte<T: Into<u8>>(&mut self, byte: T) {
        let line = self.current.line();
        self.current_chunk().add_byte(byte.into(), line);
    }

    fn emit_byte_pair<T1: Into<u8>, T2: Into<u8>>(&mut self, byte1: T1, byte2: T2) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(OpCode::Loop);

        let offset = self.current_chunk().code.len() - loop_start + 2;
        if offset > u16::MAX as usize {
            error(
                self.previous,
                "Loop body too large.",
                &mut self.had_error,
                &mut self.panic_mode,
            );
        }

        self.emit_byte((offset >> 8 & 0xFF) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn emit_jump(&mut self, op: OpCode) -> usize {
        self.emit_byte(op);
        self.emit_byte(0xFF);
        self.emit_byte(0xFF);
        return self.current_chunk().code.len() - 2;
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.current_chunk().add_constant(value);
        if constant > 255 {
            error(
                self.current,
                "too many constants in one chunk.",
                &mut self.had_error,
                &mut self.panic_mode,
            );
            return 0;
        }
        constant as u8
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_byte_pair(OpCode::Constant, constant);
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.current_chunk().code.len() - offset - 2;
        if jump > u16::MAX as usize {
            error(
                self.previous,
                "Too much code to jump over.",
                &mut self.had_error,
                &mut self.panic_mode,
            );
        }

        self.current_chunk().code[offset] = ((jump >> 8) & 0xFF) as u8;
        self.current_chunk().code[offset + 1] = (jump & 0xFF) as u8;
    }

    fn number(&mut self, _: bool) {
        let value = Value::Number(self.previous.as_str().parse::<f64>().unwrap());
        self.emit_constant(value);
    }

    fn literal(&mut self, _: bool) {
        match self.previous.kind() {
            TokenKind::False => self.emit_byte(OpCode::False),
            TokenKind::True => self.emit_byte(OpCode::True),
            TokenKind::Nil => self.emit_byte(OpCode::Nil),
            _ => unreachable!(),
        }
    }

    fn string(&mut self, _: bool) {
        let string = self.previous.as_str();
        let value = copy_string(
            string.trim_start_matches('"').trim_end_matches('"'),
            self.vm,
            Some(&mut self.compiler),
        );
        let index = self.current_chunk().add_constant(value);
        self.emit_byte_pair(OpCode::Constant, index as u8);
    }

    fn resolve_local(&mut self, name: &str) -> Option<u8> {
        self.compiler.resolve_local(
            name,
            self.previous,
            &mut self.had_error,
            &mut self.panic_mode,
        )
    }

    fn resolve_upvalue(&mut self, name: &str) -> Option<u8> {
        self.compiler.resolve_upvalue(
            name,
            self.previous,
            &mut self.had_error,
            &mut self.panic_mode,
        )
    }

    fn named_variable(&mut self, can_assign: bool) {
        let get_op: OpCode;
        let set_op: OpCode;
        let name = self.previous.as_str();
        let arg = self.resolve_local(name);
        let arg = if arg.is_some() {
            get_op = OpCode::GetLocal;
            set_op = OpCode::SetLocal;
            arg.unwrap()
        } else if let Some(arg) = self.resolve_upvalue(name) {
            get_op = OpCode::GetUpvalue;
            set_op = OpCode::SetUpvalue;
            arg
        } else {
            let constant = self.identifier_constant();
            get_op = OpCode::GetGlobal;
            set_op = OpCode::SetGlobal;
            constant
        };
        if can_assign && self.match_token(TokenKind::Equal) {
            self.expression();
            self.emit_byte_pair(set_op, arg);
        } else {
            self.emit_byte_pair(get_op, arg);
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(can_assign);
    }

    fn grouping(&mut self, _: bool) {
        self.expression();
        self.consume(TokenKind::RightParen, "Expected ')' after expression.");
    }

    fn unary(&mut self, _: bool) {
        let operator_kind = self.previous.kind();
        self.parse_precedence(Precedence::Unary);

        match operator_kind {
            TokenKind::Minus => self.emit_byte(OpCode::Negate),
            TokenKind::Bang => self.emit_byte(OpCode::Not),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self, _: bool) {
        let operator_kind = self.previous.kind();
        let parse_rule = get_rule(operator_kind);
        self.parse_precedence(parse_rule.precedence.next());

        match operator_kind {
            TokenKind::Plus => self.emit_byte(OpCode::Add),
            TokenKind::Minus => self.emit_byte(OpCode::Subtract),
            TokenKind::Star => self.emit_byte(OpCode::Multiply),
            TokenKind::Slash => self.emit_byte(OpCode::Divide),
            TokenKind::BangEqual => self.emit_byte_pair(OpCode::Equal, OpCode::Not),
            TokenKind::EqualEqual => self.emit_byte(OpCode::Equal),
            TokenKind::Greater => self.emit_byte(OpCode::Greater),
            TokenKind::GreaterEqual => self.emit_byte_pair(OpCode::Less, OpCode::Negate),
            TokenKind::Less => self.emit_byte(OpCode::Less),
            TokenKind::LessEqual => self.emit_byte_pair(OpCode::Greater, OpCode::Negate),
            _ => unreachable!(),
        }
    }

    fn argument_list(&mut self) -> u8 {
        let mut arg_count = 0;
        'arguments: while !self.check(TokenKind::RightParen) {
            self.expression();
            if arg_count == 255 {
                error(
                    self.previous,
                    "Can't have more than 255 arguments.",
                    &mut self.had_error,
                    &mut self.panic_mode,
                );
            }
            arg_count += 1;
            if !self.match_token(TokenKind::Comma) {
                break 'arguments;
            }
        }
        self.consume(TokenKind::RightParen, "Expect ')' after arguments.");
        arg_count
    }

    fn call(&mut self, _: bool) {
        let arg_count = self.argument_list();
        self.emit_byte_pair(OpCode::Call, arg_count);
    }

    fn dot(&mut self, can_assign: bool) {
        self.consume(TokenKind::Identifier, "Expect property name after '.'.");
        let name = self.identifier_constant();

        if can_assign && self.match_token(TokenKind::Equal) {
            self.expression();
            self.emit_byte_pair(OpCode::SetProperty, name);
        }
        else {
            self.emit_byte_pair(OpCode::GetProperty, name);
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let can_assign = precedence <= Precedence::Assignment;
        let prefix_rule = get_rule(self.previous.kind()).prefix;
        match prefix_rule {
            None => error(
                self.previous,
                "Expect expression.",
                &mut self.had_error,
                &mut self.panic_mode,
            ),
            Some(prefix_rule) => prefix_rule(self, can_assign),
        }

        while precedence <= get_rule(self.current.kind()).precedence {
            self.advance();
            let infix_rule = get_rule(self.previous.kind()).infix;
            match infix_rule {
                None => {
                    println!("no infix rule for {:?}", self.previous.kind());
                }
                Some(infix_rule) => infix_rule(self, can_assign),
            }

            if can_assign && self.match_token(TokenKind::Equal) {
                error(
                    self.previous,
                    "Invalid assignment target.",
                    &mut self.had_error,
                    &mut self.panic_mode,
                );
            }
        }
    }

    fn identifier_constant(&mut self) -> u8 {
        let str_obj = Object::new_string(self.previous.as_str(), self.vm, Some(&mut self.compiler));
        return self
            .current_chunk()
            .add_constant(Value::Obj(str_obj as *mut Object)) as u8;
    }

    fn add_local(&mut self, name: &'a str) {
        if self.compiler.local_count == (u8::MAX as usize) + 1 {
            error(
                self.previous,
                "Too many local variables in function.",
                &mut self.had_error,
                &mut self.panic_mode,
            );
            return;
        }
        let local = &mut self.compiler.locals[self.compiler.local_count as usize];
        self.compiler.local_count += 1;
        *local = Local::new(name, None);
    }

    fn declare_variable(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }

        let name = self.previous.as_str();
        for i in (0..self.compiler.local_count).rev() {
            let local = &self.compiler.locals[i];
            if local.depth.is_some() && local.depth < Some(self.compiler.scope_depth) {
                break;
            }
            if name == local.name {
                error(
                    self.previous,
                    "Already a variable with this name in this scope.",
                    &mut self.had_error,
                    &mut self.panic_mode,
                );
            }
        }
        self.add_local(name);
    }

    fn mark_initialized(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }
        self.compiler.locals[self.compiler.local_count - 1].depth = Some(self.compiler.scope_depth);
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TokenKind::Identifier, error_message);
        self.declare_variable();
        if self.compiler.scope_depth > 0 {
            return 0;
        }

        return self.identifier_constant();
    }

    fn define_variable(&mut self, global: u8) {
        if self.compiler.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_byte_pair(OpCode::DefineGlobal, global);
    }

    fn and(&mut self, _: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.parse_precedence(Precedence::And);
        self.patch_jump(end_jump);
    }

    fn or(&mut self, _: bool) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(else_jump);
        self.emit_byte(OpCode::Pop);
        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print);
    }

    fn return_statement(&mut self) {
        if self.compiler.function_type == FunctionType::Script {
            error(
                self.previous,
                "Can't return from top-level code.",
                &mut self.had_error,
                &mut self.panic_mode,
            );
        }

        if self.match_token(TokenKind::Semicolon) {
            self.emit_return();
        } else {
            self.expression();
            self.consume(TokenKind::Semicolon, "Expect ';' after return value.");
            self.emit_byte(OpCode::Return);
        }
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk().code.len();
        self.consume(TokenKind::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after condition.");
        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.statement();
        self.emit_loop(loop_start);
        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::Pop);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop);
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenKind::LeftParen, "Expect '(' after 'for'.");
        if self.match_token(TokenKind::Semicolon) {
        } else if self.match_token(TokenKind::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.current_chunk().code.len();
        let exit_jump = if !self.match_token(TokenKind::Semicolon) {
            self.expression();
            self.consume(TokenKind::Semicolon, "Expect ';' after loop condition.");

            Some(self.emit_jump(OpCode::JumpIfFalse))
        } else {
            None
        };
        if !self.match_token(TokenKind::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.current_chunk().code.len();
            self.expression();
            self.emit_byte(OpCode::Pop);
            self.consume(TokenKind::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        match exit_jump {
            Some(exit_jump) => {
                self.patch_jump(exit_jump);
                self.emit_byte(OpCode::Pop);
            }
            _ => (),
        }
        self.end_scope();
    }

    fn if_statement(&mut self) {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after condition.");
        let then_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.statement();

        let else_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop);

        if self.match_token(TokenKind::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn block(&mut self) {
        while !self.scanner.is_at_end() && !self.check(TokenKind::RightBrace) {
            self.declaration();
        }
        self.consume(TokenKind::RightBrace, "Expect '}' after block.");
    }

    fn function(&mut self, fun_type: FunctionType) {
        let compiler = Compiler::new(self.vm, Some(self.previous));
        let mut old_compiler = std::mem::replace(&mut self.compiler, compiler);
        self.compiler.enclosing = &mut old_compiler as *mut _;
        self.begin_scope();
        self.consume(TokenKind::LeftParen, "Expect '(' after function name.");
        'parameters: while !self.check(TokenKind::RightParen) {
            let arity = &mut unsafe { &mut *self.compiler.function }.arity;
            *arity += 1;
            if *arity > 255 {
                error(
                    self.current,
                    "Can't have more than 255 parameters.",
                    &mut self.had_error,
                    &mut self.panic_mode,
                );
            }
            let constant = self.parse_variable("Expect parameter name.");
            self.define_variable(constant);
            if !self.match_token(TokenKind::Comma) {
                break 'parameters;
            }
        }
        self.consume(TokenKind::RightParen, "Expect ')' after parameters.");
        self.consume(TokenKind::LeftBrace, "Expect '{' before function body.");
        self.block();

        let function = self.end();
        let compiler = std::mem::replace(&mut self.compiler, old_compiler);

        let f = self.make_constant(Value::Obj(function as *mut Object));
        self.emit_byte_pair(OpCode::Closure, f);

        for i in 0..unsafe { &*function }.upvalue_count {
            self.emit_byte(if compiler.upvalues[i].is_local { 1 } else { 0 });
            self.emit_byte(compiler.upvalues[i].index);
        }
    }

    fn class_declaration(&mut self) {
        self.consume(TokenKind::Identifier, "Expect class name.");
        let name_constant = self.identifier_constant();
        self.declare_variable();

        self.emit_byte_pair(OpCode::Class, name_constant);
        self.define_variable(name_constant);

        self.consume(TokenKind::LeftBrace, "Expect '{' before class body.");
        self.consume(TokenKind::RightBrace, "Expect '}' after class body.");
    }

    fn fun_declaration(&mut self) {
        let global = self.parse_variable("Expect function name.");
        self.mark_initialized();
        self.function(FunctionType::Function);
        self.define_variable(global);
    }

    fn statement(&mut self) {
        if self.match_token(TokenKind::Print) {
            self.print_statement();
        } else if self.match_token(TokenKind::If) {
            self.if_statement();
        } else if self.match_token(TokenKind::Return) {
            self.return_statement();
        } else if self.match_token(TokenKind::While) {
            self.while_statement();
        } else if self.match_token(TokenKind::For) {
            self.for_statement();
        } else if self.match_token(TokenKind::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        'sync: while !self.scanner.is_at_end() {
            if self.previous.kind() == TokenKind::Semicolon {
                return;
            }
            match self.current.kind() {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => break 'sync,
                _ => (),
            }
            self.advance();
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_token(TokenKind::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil);
        }

        self.consume(
            TokenKind::Semicolon,
            "Expect ';' after variable declaration.",
        );

        self.define_variable(global);
    }

    fn declaration(&mut self) {
        if self.match_token(TokenKind::Class) {
            self.class_declaration();
        }
        else if self.match_token(TokenKind::Var) {
            self.var_declaration();
        } else if self.match_token(TokenKind::Fun) {
            self.fun_declaration();
        } else {
            self.statement();
        }
        if self.panic_mode {
            self.synchronize();
        }
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Nil);
        self.emit_byte(OpCode::Return);
    }

    fn end(&mut self) -> *const ObjFunction {
        self.emit_return();
        self.compiler.function
    }

    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler.scope_depth -= 1;

        while self.compiler.local_count > 0
            && self.compiler.locals[self.compiler.local_count - 1].depth
                > Some(self.compiler.scope_depth)
        {
            if self.compiler.locals[self.compiler.local_count - 1].is_captured {
                self.emit_byte(OpCode::CloseUpvalue);
            } else {
                self.emit_byte(OpCode::Pop);
            }
            self.compiler.local_count -= 1;
        }
    }
}

pub fn compile<'a>(source: &str, vm: &'a mut VM) -> Result<*const ObjFunction, InterpretError> {
    let mut parser = Parser::new(source, vm);
    parser.advance();
    while !parser.scanner.is_at_end() {
        parser.declaration();
    }
    let function = parser.end();
    match parser.had_error {
        false => {
            #[cfg(debug_assertions)]
            parser.current_chunk().disassemble();
            Ok(function)
        }
        true => Err(InterpretError::Compile),
    }
}
