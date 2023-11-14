#[derive(Copy, Clone, Debug)]
pub struct CastError;

pub enum ValueType {
    Nil,
    Bool,
    Number,
    String,
    Upvalue,
    Function,
    Closure,
    Class,
    Instance,
    BoundMethod,
    Native,
}

#[cfg(not(nan_boxing))]
pub mod value {
    use super::CastError;
    use super::ValueType;
    use crate::gc::{Gc, Trace};
    use crate::object::*;
    use std::fmt::{Display, Formatter};
    #[derive(Clone, PartialEq)]
    pub enum Value {
        Nil,
        Bool(bool),
        Number(f64),
        String(Gc<ObjString>),
        _Upvalue(Gc<ObjUpvalue>),
        Function(Gc<ObjFunction>),
        Closure(Gc<ObjClosure>),
        Class(Gc<ObjClass>),
        Instance(Gc<ObjInstance>),
        BoundMethod(Gc<ObjBoundMethod>),
        Native(Gc<ObjNative>),
    }

    impl Display for Value {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            match self {
                Self::Nil => write!(f, "nil"),
                Self::Bool(b) => write!(f, "{}", b),
                Self::Number(num) => write!(f, "{}", num),
                Self::String(string) => string.borrow().fmt(f),
                Self::_Upvalue(upvalue) => upvalue.borrow().fmt(f),
                Self::Function(function) => function.borrow().fmt(f),
                Self::Closure(closure) => closure.borrow().fmt(f),
                Self::Class(class) => class.borrow().fmt(f),
                Self::Instance(instance) => instance.borrow().fmt(f),
                Self::BoundMethod(bound_method) => bound_method.borrow().fmt(f),
                Self::Native(native) => native.borrow().fmt(f),
            }
        }
    }

    impl Value {
        pub fn to_string(&self) -> Option<String> {
            match self {
                Value::String(object) => Some(object.borrow().as_str().to_string()),
                _ => None,
            }
        }

        pub fn value_type(&self) -> ValueType {
            match self {
                Value::Nil => ValueType::Nil,
                Value::Bool(_) => ValueType::Bool,
                Value::Number(_) => ValueType::Number,
                Value::String(_) => ValueType::String,
                Value::_Upvalue(_) => ValueType::Upvalue,
                Value::Function(_) => ValueType::Function,
                Value::Closure(_) => ValueType::Closure,
                Value::Class(_) => ValueType::Class,
                Value::Instance(_) => ValueType::Instance,
                Value::BoundMethod(_) => ValueType::BoundMethod,
                Value::Native(_) => ValueType::Native,
            }
        }

        pub fn nil() -> Value {
            Value::Nil
        }

        pub fn bool_(boolean: bool) -> Value {
            Value::Bool(boolean)
        }

        pub fn number(number: f64) -> Value {
            Value::Number(number)
        }

        pub fn string(string: Gc<ObjString>) -> Value {
            Value::String(string)
        }

        pub fn _upvalue(upvalue: Gc<ObjUpvalue>) -> Value {
            Value::_Upvalue(upvalue)
        }

        pub fn function(function: Gc<ObjFunction>) -> Value {
            Value::Function(function)
        }

        pub fn closure(closure: Gc<ObjClosure>) -> Value {
            Value::Closure(closure)
        }

        pub fn class(class: Gc<ObjClass>) -> Value {
            Value::Class(class)
        }

        pub fn instance(instance: Gc<ObjInstance>) -> Value {
            Value::Instance(instance)
        }

        pub fn bound_method(bound_method: Gc<ObjBoundMethod>) -> Value {
            Value::BoundMethod(bound_method)
        }

        pub fn native(native: Gc<ObjNative>) -> Value {
            Value::Native(native)
        }

        pub fn is_number(&self) -> bool {
            match self {
                Value::Number(_) => true,
                _ => false,
            }
        }

        pub fn is_string(&self) -> bool {
            match self {
                Value::String(_) => true,
                _ => false,
            }
        }

        pub fn is_falsey(&self) -> bool {
            match self {
                Value::Nil => true,
                Value::Bool(value) => !value,
                _ => false,
            }
        }

        pub fn as_number(&self) -> Result<f64, CastError> {
            match self {
                Self::Number(value) => Ok(*value),
                _ => Err(CastError),
            }
        }

        pub fn as_string(&self) -> Result<Gc<ObjString>, CastError> {
            match self {
                Self::String(string) => Ok(string.clone()),
                _ => Err(CastError),
            }
        }

        pub fn _as_upvalue(&self) -> Result<Gc<ObjUpvalue>, CastError> {
            match self {
                Self::_Upvalue(upvalue) => Ok(upvalue.clone()),
                _ => Err(CastError),
            }
        }

        pub fn as_function(&self) -> Result<Gc<ObjFunction>, CastError> {
            match self {
                Self::Function(function) => Ok(function.clone()),
                _ => Err(CastError),
            }
        }

        pub fn as_closure(&self) -> Result<Gc<ObjClosure>, CastError> {
            match self {
                Self::Closure(closure) => Ok(closure.clone()),
                _ => Err(CastError),
            }
        }

        pub fn as_class(&self) -> Result<Gc<ObjClass>, CastError> {
            match self {
                Self::Class(class) => Ok(class.clone()),
                _ => Err(CastError),
            }
        }

        pub fn as_instance(&self) -> Result<Gc<ObjInstance>, CastError> {
            match self {
                Self::Instance(instance) => Ok(instance.clone()),
                _ => Err(CastError),
            }
        }

        pub fn as_bound_method(&self) -> Result<Gc<ObjBoundMethod>, CastError> {
            match self {
                Self::BoundMethod(bound_method) => Ok(bound_method.clone()),
                _ => Err(CastError),
            }
        }

        pub fn as_native(&self) -> Result<Gc<ObjNative>, CastError> {
            match self {
                Self::Native(native) => Ok(native.clone()),
                _ => Err(CastError),
            }
        }
    }

    unsafe impl Trace for Value {
        fn trace(&self) {
            match self {
                Value::String(string) => string.trace(),
                Value::_Upvalue(upvalue) => upvalue.trace(),
                Value::Function(function) => function.trace(),
                Value::Closure(closure) => closure.trace(),
                Value::Class(class) => class.trace(),
                Value::Instance(instance) => instance.trace(),
                Value::BoundMethod(bound_method) => bound_method.trace(),
                Value::Native(native) => native.trace(),
                _ => (),
            }
        }

        fn root(&self) {
            match self {
                Value::String(string) => string.root(),
                Value::_Upvalue(upvalue) => upvalue.root(),
                Value::Function(function) => function.root(),
                Value::Closure(closure) => closure.root(),
                Value::Class(class) => class.root(),
                Value::Instance(instance) => instance.root(),
                Value::BoundMethod(bound_method) => bound_method.root(),
                Value::Native(native) => native.root(),
                _ => (),
            }
        }

        fn unroot(&self) {
            match self {
                Value::String(string) => string.unroot(),
                Value::_Upvalue(upvalue) => upvalue.unroot(),
                Value::Function(function) => function.unroot(),
                Value::Closure(closure) => closure.unroot(),
                Value::Class(class) => class.unroot(),
                Value::Instance(instance) => instance.unroot(),
                Value::BoundMethod(bound_method) => bound_method.unroot(),
                Value::Native(native) => native.unroot(),
                _ => (),
            }
        }
    }

    fn create_string_value<'a>(source: String) -> Value {
        Value::String(ObjString::new(source).into())
    }

    pub fn copy_string<'a>(source: &str) -> Value {
        create_string_value(source.to_string())
    }

    pub fn concatenate_strings(a: String, b: String) -> Value {
        let mut string = a.to_string(); //need to create this allocation because HashSet's get_or_insert() method is currently unstable
        string.push_str(&b);
        create_string_value(string)
    }
}

#[cfg(nan_boxing)]
pub mod value {
    use super::CastError;
    use super::ValueType;
    use crate::gc::{Gc, Trace};
    use crate::object::*;
    use crate::vm::InterpretError;
    use std::fmt::{Display, Formatter};
    use std::mem::ManuallyDrop;

    pub const SIGN_BIT: u64 = 0x8000000000000000;
    pub const QNAN: u64 = 0x7FF8000000000000;
    pub const REAL_INDEFINITE: u64 = SIGN_BIT | QNAN;
    pub const STRING: u64 = 0 << 48;
    pub const UPVALUE: u64 = 1 << 48;
    pub const FUNCTION: u64 = 2 << 48;
    pub const CLOSURE: u64 = 3 << 48;
    pub const CLASS: u64 = 4 << 48;
    pub const INSTANCE: u64 = 5 << 48;
    pub const BOUND_METHOD: u64 = 6 << 48;
    pub const NATIVE_FN: u64 = 7 << 48;
    pub const TAG_NIL: u64 = 0x1;
    pub const TAG_TRUE: u64 = 0x2;
    pub const TAG_FALSE: u64 = 0x3;
    pub const NIL: u64 = QNAN | TAG_NIL;
    pub const TRUE: u64 = QNAN | TAG_TRUE;
    pub const FALSE: u64 = QNAN | TAG_FALSE;

    #[repr(C)]
    pub union Value {
        number: f64,
        bits: u64,
        string: ManuallyDrop<Gc<ObjString>>,
        upvalue: ManuallyDrop<Gc<ObjUpvalue>>,
        function: ManuallyDrop<Gc<ObjFunction>>,
        closure: ManuallyDrop<Gc<ObjClosure>>,
        class: ManuallyDrop<Gc<ObjClass>>,
        instance: ManuallyDrop<Gc<ObjInstance>>,
        bound_method: ManuallyDrop<Gc<ObjBoundMethod>>,
        native: ManuallyDrop<Gc<ObjNative>>,
    }

    impl Value {
        pub fn to_string(&self) -> Option<String> {
            if let Ok(string) = self.as_string() {
                Some(string.borrow().to_string())
            } else {
                None
            }
        }

        pub fn value_type(&self) -> ValueType {
            if self.is_nil() {
                ValueType::Nil
            } else if self.is_bool() {
                ValueType::Bool
            } else if self.is_object() {
                let object_tag = (unsafe { self.bits } & NATIVE_FN);
                match object_tag {
                    STRING => ValueType::String,
                    UPVALUE => ValueType::Upvalue,
                    FUNCTION => ValueType::Function,
                    CLOSURE => ValueType::Closure,
                    CLASS => ValueType::Class,
                    INSTANCE => ValueType::Instance,
                    BOUND_METHOD => ValueType::BoundMethod,
                    NATIVE_FN => ValueType::Native,
                    _ => unreachable!(),
                }
            } else {
                ValueType::Number
            }
        }

        pub fn nil() -> Value {
            Value { bits: NIL }
        }

        pub fn bool_(boolean: bool) -> Value {
            if boolean {
                Value { bits: TRUE }
            } else {
                Value { bits: FALSE }
            }
        }

        pub fn number(number: f64) -> Value {
            Value { number }
        }

        pub fn string(string: Gc<ObjString>) -> Value {
            let mut result = Value {
                string: ManuallyDrop::new(string),
            };
            unsafe { result.bits |= QNAN | SIGN_BIT | STRING };
            result
        }

        pub fn upvalue(upvalue: Gc<ObjUpvalue>) -> Value {
            let mut result = Value {
                upvalue: ManuallyDrop::new(upvalue),
            };
            unsafe { result.bits |= QNAN | SIGN_BIT | UPVALUE };
            result
        }

        pub fn function(function: Gc<ObjFunction>) -> Value {
            let mut result = Value {
                function: ManuallyDrop::new(function),
            };
            unsafe { result.bits |= QNAN | SIGN_BIT | FUNCTION };
            result
        }

        pub fn closure(closure: Gc<ObjClosure>) -> Value {
            let mut result = Value {
                closure: ManuallyDrop::new(closure),
            };
            unsafe { result.bits |= QNAN | SIGN_BIT | CLOSURE };
            result
        }

        pub fn class(class: Gc<ObjClass>) -> Value {
            let mut result = Value {
                class: ManuallyDrop::new(class),
            };
            unsafe { result.bits |= QNAN | SIGN_BIT | CLASS };
            result
        }

        pub fn instance(instance: Gc<ObjInstance>) -> Value {
            let mut result = Value {
                instance: ManuallyDrop::new(instance),
            };
            unsafe { result.bits |= QNAN | SIGN_BIT | INSTANCE };
            result
        }

        pub fn bound_method(bound_method: Gc<ObjBoundMethod>) -> Value {
            let mut result = Value {
                bound_method: ManuallyDrop::new(bound_method),
            };
            unsafe { result.bits |= QNAN | SIGN_BIT | BOUND_METHOD };
            result
        }

        pub fn native(native: Gc<ObjNative>) -> Value {
            let mut result = Value {
                native: ManuallyDrop::new(native),
            };
            unsafe { result.bits |= QNAN | SIGN_BIT | NATIVE_FN };
            result
        }

        pub fn is_object(&self) -> bool {
            unsafe {
                self.bits != REAL_INDEFINITE && self.bits & REAL_INDEFINITE == REAL_INDEFINITE
            }
        }

        pub fn is_number(&self) -> bool {
            (unsafe { self.bits } & QNAN) != QNAN
        }

        pub fn is_string(&self) -> bool {
            match self.as_string() {
                Ok(_) => true,
                Err(_) => false,
            }
        }

        pub fn is_falsey(&self) -> bool {
            unsafe {
                if self.bits == FALSE || self.bits == NIL {
                    true
                } else {
                    false
                }
            }
        }

        pub fn is_bool(&self) -> bool {
            unsafe { return self.bits == TRUE || self.bits == FALSE }
        }

        pub fn is_nil(&self) -> bool {
            unsafe { return self.bits == NIL }
        }

        pub fn as_bool(&self) -> Result<bool, InterpretError> {
            unsafe {
                if self.bits == TRUE {
                    return Ok(true);
                } else if self.bits == FALSE {
                    return Ok(false);
                }
            }

            Err(InterpretError::Runtime)
        }

        pub fn as_number(&self) -> Result<f64, InterpretError> {
            if unsafe { self.bits } & QNAN == QNAN {
                return Err(InterpretError::Runtime);
            }

            Ok(unsafe { self.number })
        }

        pub fn as_string(&self) -> Result<Gc<ObjString>, CastError> {
            if !self.is_object() {
                return Err(CastError::NotAnObject);
            } else if unsafe { self.bits } & NATIVE_FN != STRING {
                return Err(CastError::IncorrectObjectType);
            }
            //bit weird, but temp will not cause a drop of self, and dereferencing then cloning
            //ManuallyDrop<Gc<T>> makes a normal Gc<T>
            let temp = Value {
                bits: unsafe { self.bits } & !(QNAN | SIGN_BIT | NATIVE_FN),
            };
            let result = unsafe { (*temp.string).clone() };
            std::mem::forget(temp);
            return Ok(result);
        }

        pub fn as_upvalue(&self) -> Result<Gc<ObjUpvalue>, CastError> {
            if !self.is_object() {
                return Err(CastError::NotAnObject);
            } else if unsafe { self.bits } & NATIVE_FN != UPVALUE {
                return Err(CastError::IncorrectObjectType);
            }
            //bit weird, but temp will not cause a drop of self, and dereferencing then cloning
            //ManuallyDrop<Gc<T>> makes a normal Gc<T>
            let temp = Value {
                bits: unsafe { self.bits } & !(QNAN | SIGN_BIT | NATIVE_FN),
            };
            let result = unsafe { (*temp.upvalue).clone() };
            std::mem::forget(temp);
            return Ok(result);
        }

        pub fn as_function(&self) -> Result<Gc<ObjFunction>, CastError> {
            if !self.is_object() {
                return Err(CastError::NotAnObject);
            } else if unsafe { self.bits } & NATIVE_FN != FUNCTION {
                return Err(CastError::IncorrectObjectType);
            }
            //bit weird, but temp will not cause a drop of self, and dereferencing then cloning
            //ManuallyDrop<Gc<T>> makes a normal Gc<T>
            let temp = Value {
                bits: unsafe { self.bits } & !(QNAN | SIGN_BIT | NATIVE_FN),
            };
            let result = unsafe { (*temp.function).clone() };
            std::mem::forget(temp);
            return Ok(result);
        }

        pub fn as_closure(&self) -> Result<Gc<ObjClosure>, CastError> {
            if !self.is_object() {
                return Err(CastError::NotAnObject);
            } else if unsafe { self.bits } & NATIVE_FN != CLOSURE {
                return Err(CastError::IncorrectObjectType);
            }
            //bit weird, but temp will not cause a drop of self, and dereferencing then cloning
            //ManuallyDrop<Gc<T>> makes a normal Gc<T>
            let temp = Value {
                bits: unsafe { self.bits } & !(QNAN | SIGN_BIT | NATIVE_FN),
            };
            let result = unsafe { (*temp.closure).clone() };
            std::mem::forget(temp);
            return Ok(result);
        }

        pub fn as_class(&self) -> Result<Gc<ObjClass>, CastError> {
            if !self.is_object() {
                return Err(CastError::NotAnObject);
            } else if unsafe { self.bits } & NATIVE_FN != CLASS {
                return Err(CastError::IncorrectObjectType);
            }
            //bit weird, but temp will not cause a drop of self, and dereferencing then cloning
            //ManuallyDrop<Gc<T>> makes a normal Gc<T>
            let temp = Value {
                bits: unsafe { self.bits } & !(QNAN | SIGN_BIT | NATIVE_FN),
            };
            let result = unsafe { (*temp.class).clone() };
            std::mem::forget(temp);
            return Ok(result);
        }

        pub fn as_instance(&self) -> Result<Gc<ObjInstance>, CastError> {
            if !self.is_object() {
                return Err(CastError::NotAnObject);
            } else if unsafe { self.bits } & NATIVE_FN != INSTANCE {
                return Err(CastError::IncorrectObjectType);
            }
            //bit weird, but temp will not cause a drop of self, and dereferencing then cloning
            //ManuallyDrop<Gc<T>> makes a normal Gc<T>
            let temp = Value {
                bits: unsafe { self.bits } & !(QNAN | SIGN_BIT | NATIVE_FN),
            };
            let result = unsafe { (*temp.instance).clone() };
            std::mem::forget(temp);
            return Ok(result);
        }

        pub fn as_bound_method(&self) -> Result<Gc<ObjBoundMethod>, CastError> {
            if !self.is_object() {
                return Err(CastError::NotAnObject);
            } else if unsafe { self.bits } & NATIVE_FN != BOUND_METHOD {
                return Err(CastError::IncorrectObjectType);
            }
            //bit weird, but temp will not cause a drop of self, and dereferencing then cloning
            //ManuallyDrop<Gc<T>> makes a normal Gc<T>
            let temp = Value {
                bits: unsafe { self.bits } & !(QNAN | SIGN_BIT | NATIVE_FN),
            };
            let result = unsafe { (*temp.bound_method).clone() };
            std::mem::forget(temp);
            return Ok(result);
        }

        pub fn as_native(&self) -> Result<Gc<ObjNative>, CastError> {
            if !self.is_object() {
                return Err(CastError::NotAnObject);
            } else if unsafe { self.bits } & NATIVE_FN != NATIVE_FN {
                return Err(CastError::IncorrectObjectType);
            }
            //bit weird, but temp will not cause a drop of self, and dereferencing then cloning
            //ManuallyDrop<Gc<T>> makes a normal Gc<T>
            let temp = Value {
                bits: unsafe { self.bits } & !(QNAN | SIGN_BIT | NATIVE_FN),
            };
            let result = unsafe { (*temp.native).clone() };
            std::mem::forget(temp);
            return Ok(result);
        }
    }

    impl Clone for Value {
        fn clone(&self) -> Self {
            match self.value_type() {
                ValueType::String => Value::string(self.as_string().unwrap()),
                ValueType::Upvalue => Value::upvalue(self.as_upvalue().unwrap()),
                ValueType::Function => Value::function(self.as_function().unwrap()),
                ValueType::Closure => Value::closure(self.as_closure().unwrap()),
                ValueType::Class => Value::class(self.as_class().unwrap()),
                ValueType::Instance => Value::instance(self.as_instance().unwrap()),
                ValueType::BoundMethod => Value::bound_method(self.as_bound_method().unwrap()),
                ValueType::Native => Value::native(self.as_native().unwrap()),
                _ => Value {
                    bits: unsafe { self.bits },
                },
            }
        }
    }

    impl PartialEq for Value {
        fn eq(&self, other: &Self) -> bool {
            match (self.value_type(), other.value_type()) {
                (ValueType::Nil, ValueType::Nil) => true,
                (ValueType::Bool, ValueType::Bool) => {
                    self.as_bool().unwrap() == other.as_bool().unwrap()
                }
                (ValueType::Number, ValueType::Number) => {
                    self.as_number().unwrap() == other.as_number().unwrap()
                }
                (ValueType::String, ValueType::String) => {
                    self.as_string().unwrap() == other.as_string().unwrap()
                }
                _ => false,
            }
        }
    }

    impl Display for Value {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self.value_type() {
                ValueType::Nil => f.write_str("nil"),
                ValueType::Bool => self.as_bool().unwrap().fmt(f),
                ValueType::Number => self.as_number().unwrap().fmt(f),
                ValueType::String => self.as_string().unwrap().fmt(f),
                ValueType::Upvalue => self.as_upvalue().unwrap().fmt(f),
                ValueType::Function => self.as_function().unwrap().fmt(f),
                ValueType::Closure => self.as_closure().unwrap().fmt(f),
                ValueType::Class => self.as_class().unwrap().fmt(f),
                ValueType::Instance => self.as_instance().unwrap().fmt(f),
                ValueType::BoundMethod => self.as_bound_method().unwrap().fmt(f),
                ValueType::Native => self.as_native().unwrap().fmt(f),
            }
        }
    }

    unsafe impl Trace for Value {
        fn trace(&self) {
            match self.value_type() {
                ValueType::String => self.as_string().unwrap().trace(),
                ValueType::Upvalue => self.as_upvalue().unwrap().trace(),
                ValueType::Function => self.as_function().unwrap().trace(),
                ValueType::Closure => self.as_closure().unwrap().trace(),
                ValueType::Class => self.as_class().unwrap().trace(),
                ValueType::Instance => self.as_instance().unwrap().trace(),
                ValueType::BoundMethod => self.as_bound_method().unwrap().trace(),
                ValueType::Native => self.as_native().unwrap().trace(),
                _ => (),
            }
        }

        fn root(&self) {
            match self.value_type() {
                ValueType::String => self.as_string().unwrap().root(),
                ValueType::Upvalue => self.as_upvalue().unwrap().root(),
                ValueType::Function => self.as_function().unwrap().root(),
                ValueType::Closure => self.as_closure().unwrap().root(),
                ValueType::Class => self.as_class().unwrap().root(),
                ValueType::Instance => self.as_instance().unwrap().root(),
                ValueType::BoundMethod => self.as_bound_method().unwrap().root(),
                ValueType::Native => self.as_native().unwrap().root(),
                _ => (),
            }
        }

        fn unroot(&self) {
            match self.value_type() {
                ValueType::String => self.as_string().unwrap().unroot(),
                ValueType::Upvalue => self.as_upvalue().unwrap().unroot(),
                ValueType::Function => self.as_function().unwrap().unroot(),
                ValueType::Closure => self.as_closure().unwrap().unroot(),
                ValueType::Class => self.as_class().unwrap().unroot(),
                ValueType::Instance => self.as_instance().unwrap().unroot(),
                ValueType::BoundMethod => self.as_bound_method().unwrap().unroot(),
                ValueType::Native => self.as_native().unwrap().unroot(),
                _ => (),
            }
        }
    }

    impl Drop for Value {
        fn drop(&mut self) {
            match self.value_type() {
                ValueType::String => unsafe {
                    self.bits &= !(QNAN | SIGN_BIT | NATIVE_FN);
                    ManuallyDrop::drop(&mut self.string);
                },
                ValueType::Upvalue => unsafe {
                    self.bits &= !(QNAN | SIGN_BIT | NATIVE_FN);
                    ManuallyDrop::drop(&mut self.upvalue);
                },
                ValueType::Function => unsafe {
                    self.bits &= !(QNAN | SIGN_BIT | NATIVE_FN);
                    ManuallyDrop::drop(&mut self.function);
                },
                ValueType::Closure => unsafe {
                    self.bits &= !(QNAN | SIGN_BIT | NATIVE_FN);
                    ManuallyDrop::drop(&mut self.closure);
                },
                ValueType::Class => unsafe {
                    self.bits &= !(QNAN | SIGN_BIT | NATIVE_FN);
                    ManuallyDrop::drop(&mut self.class);
                },
                ValueType::Instance => unsafe {
                    self.bits &= !(QNAN | SIGN_BIT | NATIVE_FN);
                    ManuallyDrop::drop(&mut self.instance);
                },
                ValueType::BoundMethod => unsafe {
                    self.bits &= !(QNAN | SIGN_BIT | NATIVE_FN);
                    ManuallyDrop::drop(&mut self.bound_method);
                },
                ValueType::Native => unsafe {
                    self.bits &= !(QNAN | SIGN_BIT | NATIVE_FN);
                    ManuallyDrop::drop(&mut self.native);
                },
                _ => (),
            }
        }
    }

    fn create_string_value<'a>(source: String) -> Value {
        Value::string(ObjString::new(source).into())
    }

    pub fn copy_string<'a>(source: &str) -> Value {
        create_string_value(source.to_string())
    }

    pub fn concatenate_strings(a: String, b: String) -> Value {
        let mut string = a.to_string();
        string.push_str(&b);
        create_string_value(string)
    }
}
