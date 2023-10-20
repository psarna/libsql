//! Structures in Hrana that are common for WebSockets and HTTP.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, prost::Message)]
pub struct Error {
    #[prost(string, tag = "1")]
    pub message: String,
    #[prost(string, tag = "2")]
    pub code: String,
}

#[derive(Deserialize, prost::Message)]
pub struct Stmt {
    #[serde(default)]
    #[prost(string, optional, tag = "1")]
    pub sql: Option<String>,
    #[serde(default)]
    #[prost(int32, optional, tag = "2")]
    pub sql_id: Option<i32>,
    #[serde(default)]
    #[prost(message, repeated, tag = "3")]
    pub args: Vec<Value>,
    #[serde(default)]
    #[prost(message, repeated, tag = "4")]
    pub named_args: Vec<NamedArg>,
    #[serde(default)]
    #[prost(bool, optional, tag = "5")]
    pub want_rows: Option<bool>,
    #[serde(default)]
    #[prost(uint64, optional, tag = "6")]
    pub replication_index: Option<u64>,
}

#[derive(Deserialize, prost::Message)]
pub struct NamedArg {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(message, required, tag = "2")]
    pub value: Value,
}

#[derive(Serialize, prost::Message)]
pub struct StmtResultStats {
    #[prost(int32, required, tag = "1")]
    pub rows_read: i32,
    #[prost(int32, required, tag = "2")]
    pub rows_written: i32,
}

impl From<crate::query_result_builder::QueryStats> for StmtResultStats {
    fn from(stats: crate::query_result_builder::QueryStats) -> Self {
        StmtResultStats {
            rows_read: stats.rows_read,
            rows_written: stats.rows_written,
        }
    }
}

#[derive(Serialize, prost::Message)]
pub struct StmtResult {
    #[prost(message, repeated, tag = "1")]
    pub cols: Vec<Col>,
    #[prost(message, repeated, tag = "2")]
    pub rows: Vec<Row>,
    #[prost(uint64, tag = "3")]
    pub affected_row_count: u64,
    #[serde(with = "option_i64_as_str")]
    #[prost(sint64, optional, tag = "4")]
    pub last_insert_rowid: Option<i64>,
    #[prost(uint64, optional, tag = "5")]
    pub replication_index: Option<u64>,
    #[prost(message, optional, tag = "6")]
    pub stats: Option<StmtResultStats>,
}

#[derive(Serialize, prost::Message)]
pub struct Col {
    #[prost(string, optional, tag = "1")]
    pub name: Option<String>,
    #[prost(string, optional, tag = "2")]
    pub decltype: Option<String>,
}

#[derive(Serialize, prost::Message)]
#[serde(transparent)]
pub struct Row {
    #[prost(message, repeated, tag = "1")]
    pub values: Vec<Value>,
}

#[derive(Deserialize, prost::Message)]
pub struct Batch {
    #[prost(message, repeated, tag = "1")]
    pub steps: Vec<BatchStep>,
    #[prost(uint64, optional, tag = "2")]
    #[serde(default)]
    pub replication_index: Option<u64>,
}

#[derive(Deserialize, prost::Message)]
pub struct BatchStep {
    #[serde(default)]
    #[prost(message, optional, tag = "1")]
    pub condition: Option<BatchCond>,
    #[prost(message, required, tag = "2")]
    pub stmt: Stmt,
}

#[derive(Serialize, Debug, Default)]
pub struct BatchResult {
    pub step_results: Vec<Option<StmtResult>>,
    pub step_errors: Vec<Option<Error>>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BatchCond {
    #[serde(skip_deserializing)]
    #[default]
    None,
    Ok {
        step: u32,
    },
    Error {
        step: u32,
    },
    Not {
        cond: Box<BatchCond>,
    },
    And(BatchCondList),
    Or(BatchCondList),
    IsAutocommit {},
}

#[derive(Deserialize, prost::Message)]
pub struct BatchCondList {
    #[prost(message, repeated, tag = "1")]
    pub conds: Vec<BatchCond>,
}

#[derive(Serialize, Debug, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CursorEntry {
    #[serde(skip_deserializing)]
    #[default]
    None,
    StepBegin(StepBeginEntry),
    StepEnd(StepEndEntry),
    StepError(StepErrorEntry),
    Row {
        row: Row,
    },
    Error {
        error: Error,
    },
    ReplicationIndex {
        replication_index: Option<u64>,
    },
}

#[derive(Serialize, prost::Message)]
pub struct StepBeginEntry {
    #[prost(uint32, tag = "1")]
    pub step: u32,
    #[prost(message, repeated, tag = "2")]
    pub cols: Vec<Col>,
}

#[derive(Serialize, prost::Message)]
pub struct StepEndEntry {
    #[prost(uint64, tag = "1")]
    pub affected_row_count: u64,
    #[prost(sint64, optional, tag = "2")]
    pub last_insert_rowid: Option<i64>,
}

#[derive(Serialize, prost::Message)]
pub struct StepErrorEntry {
    #[prost(uint32, tag = "1")]
    pub step: u32,
    #[prost(message, required, tag = "2")]
    pub error: Error,
}

#[derive(Serialize, prost::Message)]
pub struct DescribeResult {
    #[prost(message, repeated, tag = "1")]
    pub params: Vec<DescribeParam>,
    #[prost(message, repeated, tag = "2")]
    pub cols: Vec<DescribeCol>,
    #[prost(bool, tag = "3")]
    pub is_explain: bool,
    #[prost(bool, tag = "4")]
    pub is_readonly: bool,
}

#[derive(Serialize, prost::Message)]
pub struct DescribeParam {
    #[prost(string, optional, tag = "1")]
    pub name: Option<String>,
}

#[derive(Serialize, prost::Message)]
pub struct DescribeCol {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, optional, tag = "2")]
    pub decltype: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Value {
    #[serde(skip_deserializing)]
    #[default]
    None,
    Null,
    Integer {
        #[serde(with = "i64_as_str")]
        value: i64,
    },
    Float {
        value: f64,
    },
    Text {
        value: Arc<str>,
    },
    Blob {
        #[serde(with = "bytes_as_base64", rename = "base64")]
        value: Bytes,
    },
}

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
    use serde::{ser, Serialize as _};

    pub fn serialize<S: ser::Serializer>(value: &Option<i64>, ser: S) -> Result<S::Ok, S::Error> {
        value.map(|v| v.to_string()).serialize(ser)
    }
}

mod bytes_as_base64 {
    use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
    use bytes::Bytes;
    use serde::{de, ser};
    use serde::{de::Error as _, Serialize as _};

    pub fn serialize<S: ser::Serializer>(value: &Bytes, ser: S) -> Result<S::Ok, S::Error> {
        STANDARD_NO_PAD.encode(value).serialize(ser)
    }

    pub fn deserialize<'de, D: de::Deserializer<'de>>(de: D) -> Result<Bytes, D::Error> {
        let text = <&'de str as de::Deserialize>::deserialize(de)?;
        let text = text.trim_end_matches('=');
        let bytes = STANDARD_NO_PAD.decode(text).map_err(|_| {
            D::Error::invalid_value(de::Unexpected::Str(text), &"binary data encoded as base64")
        })?;
        Ok(Bytes::from(bytes))
    }
}
