#![allow(dead_code)]
//! Simple parser for [Telegram Bot API docs]
//!
//! [Telegram Bot API docs]: https://core.telegram.org/bots/api
#[macro_use]
extern crate strum_macros;

use select::document::Document;
use select::predicate::{Name};
use std::fs::File;
use std::any::type_name;


#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Schema {
    pub recent_changes: Vec<Change>, // TODO: changes
    pub primitives: &'static [&'static str],
    pub types: Vec<Type>,
    pub methods: Vec<Method>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Change {
    pub date: String, // TODO
    pub version: String, // TODO
    pub changes: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Type {
    pub name: String,
    pub descr: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Primitive,
    pub descr: String,
}

pub const PRIMITIVES: &'static [&'static str] = Primitive::VARIANTS;

#[derive(Debug, PartialEq, Eq, Hash, Clone, EnumVariantNames)]
pub enum Primitive {
    /// tg's Integer
    I32,
    /// tg's Integer in some cases (writers of the doc, I hate you)
    I64,
    /// tg's Float
    F32,
    /// tg's String
    String,
    /// tg's Boolean
    Bool,
    /// tg's True
    True,
    /// Struct type
    Struct { name: String },
    /// tg's `Array of`
    Array(Box<Primitive>),
    /// tg's "Optional. " in description
    Option(Box<Primitive>),

    // Special types:
    /// tg's Integer or String
    ChatId,
    /// Parse mode is String in docs, but it's actually `HTML | Markdown` sum type
    ParseMode,
    /// Magical type, see https://core.telegram.org/bots/api#sending-files
    /// Smt like `FileId(String) | Url(String) | File(...)` sum type
    /// (do not forget that file must be uploaded with multipart/form-data)
    InputFile,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Method {
    pub name: String,
    pub descr: String,
    pub params: Vec<Param>,
    pub return_ty: Primitive,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Primitive,
    pub required: Required,
    pub descr: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Required {
    Yes, Optional
}

/// First step - parse HTML
pub mod parse;

/// Second step - clean parsed HTML and convert to typed structures
pub mod transform;