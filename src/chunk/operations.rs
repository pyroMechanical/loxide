#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum OpCode {
    Constant = 0,
    Nil,
    True,
    False,
    Pop,
    GetLocal,
    SetLocal,
    GetGlobal,
    DefineGlobal,
    SetGlobal,
    GetUpvalue,
    SetUpvalue,
    GetProperty,
    SetProperty,
    GetSuper,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,
    Jump,
    JumpIfFalse,
    Loop,
    Call,
    Invoke,
    SuperInvoke,
    Closure,
    CloseUpvalue,
    Class,
    Inherit,
    Method,
    Return,
}
impl TryInto<OpCode> for u8 {
    type Error = ();
    fn try_into(self) -> Result<OpCode, Self::Error> {
        if self > OpCode::Return as u8 {
            Err(())
        } else {
            Ok(unsafe { std::mem::transmute(self) })
        }
    }
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        unsafe { std::mem::transmute(self) }
    }
}
