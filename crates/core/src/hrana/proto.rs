//! Messages in the Hrana protocol.
//!
//! Please consult the Hrana specification in the `docs/` directory for more information.
#![allow(dead_code)]

use std::collections::VecDeque;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Hello { jwt: Option<String> },
    Request { request_id: i32, request: Request },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    HelloOk {},
    HelloError { error: Error },
    ResponseOk { request_id: i32, response: Response },
    ResponseError { request_id: i32, error: Error },
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    OpenStream(OpenStreamReq),
    CloseStream(CloseStreamReq),
    Execute(ExecuteReq),
    Batch(BatchReq),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    OpenStream(OpenStreamResp),
    CloseStream(CloseStreamResp),
    Execute(ExecuteResp),
    Batch(BatchResp),
}

#[derive(Serialize, Debug)]
pub struct OpenStreamReq {
    pub stream_id: i32,
}

#[derive(Deserialize, Debug)]
pub struct OpenStreamResp {}

#[derive(Serialize, Debug)]
pub struct CloseStreamReq {
    pub stream_id: i32,
}

#[derive(Deserialize, Debug)]
pub struct CloseStreamResp {}

#[derive(Serialize, Debug)]
pub struct ExecuteReq {
    pub stream_id: i32,
    pub stmt: Stmt,
}

#[derive(Deserialize, Debug)]
pub struct ExecuteResp {
    pub result: StmtResult,
}

#[derive(Serialize, Debug, Clone)]
pub struct Stmt {
    pub sql: String,
    #[serde(default)]
    pub args: Vec<Value>,
    #[serde(default)]
    pub named_args: Vec<NamedArg>,
    pub want_rows: bool,
}

impl Stmt {
    pub fn new(sql: impl Into<String>, want_rows: bool) -> Self {
        let sql = sql.into();
        Self {
            sql,
            want_rows,
            named_args: Vec::new(),
            args: Vec::new(),
        }
    }

    pub fn bind(&mut self, val: Value) {
        self.args.push(val);
    }

    pub fn bind_named(&mut self, name: String, value: Value) {
        self.named_args.push(NamedArg { name, value });
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct NamedArg {
    pub name: String,
    pub value: Value,
}

#[derive(Deserialize, Clone, Debug)]
pub struct StmtResult {
    pub cols: Vec<Col>,
    pub rows: VecDeque<Vec<Value>>,
    pub affected_row_count: u64,
    #[serde(with = "option_i64_as_str")]
    pub last_insert_rowid: Option<i64>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Col {
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Value {
    Null,
    Integer {
        #[serde(with = "i64_as_str")]
        value: i64,
    },
    Float {
        value: f64,
    },
    Text {
        value: String,
    },
    Blob {
        #[serde(with = "bytes_as_base64", rename = "base64")]
        value: Vec<u8>,
    },
}

#[derive(Serialize, Debug)]
pub struct BatchReq {
    pub stream_id: i32,
    pub batch: Batch,
}

#[derive(Serialize, Debug, Default)]
pub struct Batch {
    steps: Vec<BatchStep>,
}

impl Batch {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn step(&mut self, condition: Option<BatchCond>, stmt: Stmt) {
        self.steps.push(BatchStep { condition, stmt });
    }
}

#[derive(Serialize, Debug)]
pub struct BatchStep {
    condition: Option<BatchCond>,
    stmt: Stmt,
}

#[derive(Serialize, Debug)]
pub enum BatchCond {
    Ok { step: i32 },
    Error { step: i32 },
    Not { cond: Box<BatchCond> },
    And { conds: Vec<BatchCond> },
    Or { conds: Vec<BatchCond> },
}

#[derive(Deserialize, Debug)]
pub struct BatchResp {
    pub result: BatchResult,
}

#[derive(Deserialize, Debug)]
pub struct BatchResult {
    pub step_results: Vec<Option<StmtResult>>,
    pub step_errors: Vec<Option<Error>>,
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            None => Self::Null,
            Some(t) => t.into(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Error {
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

mod i64_as_str {
    use serde::{de, ser};
    use serde::{de::Error as _, Serialize as _};

    pub fn serialize<S: ser::Serializer>(value: &i64, ser: S) -> Result<S::Ok, S::Error> {
        value.to_string().serialize(ser)
    }

    pub fn deserialize<'de, D: de::Deserializer<'de>>(de: D) -> Result<i64, D::Error> {
        let str_value = <&'de str as de::Deserialize>::deserialize(de)?;
        str_value.parse().map_err(|_| {
            D::Error::invalid_value(
                de::Unexpected::Str(str_value),
                &"decimal integer as a string",
            )
        })
    }
}

mod option_i64_as_str {
    use serde::{de, de::Error as _};

    pub fn deserialize<'de, D: de::Deserializer<'de>>(de: D) -> Result<Option<i64>, D::Error> {
        let str_value = <Option<&'de str> as de::Deserialize>::deserialize(de)?;
        str_value
            .map(|s| {
                s.parse().map_err(|_| {
                    D::Error::invalid_value(de::Unexpected::Str(s), &"decimal integer as a string")
                })
            })
            .transpose()
    }
}

mod bytes_as_base64 {
    use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
    use serde::{de, ser};
    use serde::{de::Error as _, Serialize as _};

    pub fn serialize<S: ser::Serializer>(value: &Vec<u8>, ser: S) -> Result<S::Ok, S::Error> {
        STANDARD_NO_PAD.encode(value).serialize(ser)
    }

    pub fn deserialize<'de, D: de::Deserializer<'de>>(de: D) -> Result<Vec<u8>, D::Error> {
        let str_value = <&'de str as de::Deserialize>::deserialize(de)?;
        STANDARD_NO_PAD
            .decode(str_value.trim_end_matches('='))
            .map_err(|_| {
                D::Error::invalid_value(
                    de::Unexpected::Str(str_value),
                    &"binary data encoded as base64",
                )
            })
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Integer { value: n } => write!(f, "{n}"),
            Value::Float { value: d } => write!(f, "{d}"),
            Value::Text { value: s } => write!(f, "{}", serde_json::json!(s)),
            Value::Blob { value: b } => {
                use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
                let b = BASE64_STANDARD_NO_PAD.encode(b);
                write!(f, "{{\"base64\": {b}}}")
            }
        }
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Value {
        Value::Null
    }
}

macro_rules! impl_from_value {
    ($typename: ty, $variant: ident) => {
        impl From<$typename> for Value {
            fn from(t: $typename) -> Value {
                Value::$variant { value: t.into() }
            }
        }
    };
}

impl_from_value!(String, Text);
impl_from_value!(&String, Text);
impl_from_value!(&str, Text);

impl_from_value!(i8, Integer);
impl_from_value!(i16, Integer);
impl_from_value!(i32, Integer);
impl_from_value!(i64, Integer);

impl_from_value!(u8, Integer);
impl_from_value!(u16, Integer);
impl_from_value!(u32, Integer);

// rust doesn't like usize.into() for i64, so do it manually.
impl From<usize> for Value {
    fn from(t: usize) -> Value {
        Value::Integer { value: t as _ }
    }
}

impl From<isize> for Value {
    fn from(t: isize) -> Value {
        Value::Integer { value: t as _ }
    }
}

impl_from_value!(f32, Float);
impl_from_value!(f64, Float);

impl_from_value!(Vec<u8>, Blob);

macro_rules! impl_value_try_from_core {
    ($variant: ident, $typename: ty) => {
        impl TryFrom<Value> for $typename {
            type Error = String;
            fn try_from(v: Value) -> Result<$typename, Self::Error> {
                match v {
                    Value::$variant { value: v } => v.try_into().map_err(|e| format!("{e}")),
                    other => Err(format!(
                        "cannot transform {other:?} to {}",
                        stringify!($variant)
                    )),
                }
            }
        }
    };
}

macro_rules! impl_value_try_from_pod {
    ($variant: ident, $typename: ty) => {
        impl_value_try_from_core!($variant, $typename);

        impl TryFrom<&Value> for $typename {
            type Error = String;
            fn try_from(v: &Value) -> Result<$typename, Self::Error> {
                match v {
                    Value::$variant { value: v } => (*v).try_into().map_err(|e| format!("{e}")),
                    other => Err(format!(
                        "cannot transform {other:?} to {}",
                        stringify!($variant)
                    )),
                }
            }
        }
    };
}

macro_rules! impl_value_try_from_ref {
    ($variant: ident, $typename: ty, $reftype: ty) => {
        impl_value_try_from_core!($variant, $typename);

        impl<'a> TryFrom<&'a Value> for &'a $reftype {
            type Error = String;
            fn try_from(v: &'a Value) -> Result<&'a $reftype, Self::Error> {
                match v {
                    Value::$variant { value: v } => Ok(v),
                    other => Err(format!(
                        "cannot transform {other:?} to {}",
                        stringify!($variant)
                    )),
                }
            }
        }
    };
}

impl_value_try_from_pod!(Integer, i8);
impl_value_try_from_pod!(Integer, i16);
impl_value_try_from_pod!(Integer, i32);
impl_value_try_from_pod!(Integer, i64);
impl_value_try_from_pod!(Integer, u8);
impl_value_try_from_pod!(Integer, u16);
impl_value_try_from_pod!(Integer, u32);
impl_value_try_from_pod!(Integer, u64);
impl_value_try_from_pod!(Integer, usize);
impl_value_try_from_pod!(Integer, isize);
impl_value_try_from_pod!(Float, f64);

impl_value_try_from_ref!(Text, String, str);
impl_value_try_from_ref!(Blob, Vec<u8>, [u8]);
