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
pub mod step1 {
    use select::document::Document;
    use select::predicate::Name;
    use itertools::Itertools;

    /// Raw type specification from docs.
    #[derive(Debug)]
    pub struct TypeSpecification {
        /// Name of the type
        pub h4: String,
        /// Description of the type
        pub p: Option<String>,
        /// Fields of the type
        pub table: Vec<Vec<String>>,
    }

    /// Parse all types from ["Available types"] section in [telegram docs]
    /// into raw [TypeSpecification]
    ///
    /// ["Available types"]: https://core.telegram.org/bots/api#available-types
    /// [telegram docs]: https://core.telegram.org/bots/api
    /// [TypeSpecification]: self::TypeSpecification
    pub fn parse_available_types(doc: Document) -> Vec<TypeSpecification> {
        let start = doc
            .find(Name("h3"))
            .find(|n| n.text() == "Available types")
            .unwrap()
            .index();

        let stop = doc
            .find(Name("h4"))
            // InputFile has a special doc and can't be parsed with this function
            .find(|n| n.text() == "InputFile")
            .unwrap()
            .index();

        let res = (start..stop)
            .map(|i| doc.nth(i))
            .while_some()
            .fold(Vec::<TypeSpecification>::new(), |mut acc, node| {
                if node.is(Name("h4")) {
                    // All <h4> in our interval is type names
                    // So them we meet <h4>, it's new type definition
                    acc.push(TypeSpecification { h4: node.text(), p: None, table: vec![] });
                } else if node.is(Name("p")) {
                    let last = acc.last_mut();
                    let text = node.text();

                    match last {
                        // Last type definition exists, but has no description
                        Some(TypeSpecification { p: p @ None, .. }) => { *p = Some(text) },
                        // Last type definition exists, and has description
                        Some(TypeSpecification { p: Some(t), .. }) => { t.push_str(&text) },
                        // Skip all <p> which are before first type definition
                        None => println!("Warn: skipped <p>: {}", node.text()),
                    };
                } else if node.is(Name("table")) {
                    let vec: Vec<Vec<String>> = (node.index()..) // start from node
                        .map(|i| doc.nth(i)) // map to elements
                        .while_some() // stop on first `None`
                        .take_while(|tag| !tag.is(Name("h4"))) // stop on first h4
                        .skip_while(|tag| !tag.is(Name("table"))) // idk
                        .filter(|tag| tag.is(Name("tr"))) // filter <tr> tags
                        /* iter other <tr> tags */
                        .map(|n| n
                            .children()
                            // filter <td> tags that are children of <tr>
                            .filter(|tag| tag.is(Name("td")))
                            .map(|tag| tag.text())
                            .collect::<Vec<_>>()
                        )
                        .filter(|v| !v.is_empty())
                        .collect::<Vec<_>>();



                    let last = acc.last_mut().expect("h4 before table");

                    let TypeSpecification { table: v, .. } = last;
                    v.extend(vec);
                }

                acc
            });

        res
    }

}

/// Second step - clean parsed HTML and convert to typed structures
pub mod step2 {
    use super::{step1, Type};
    use crate::step1::TypeSpecification;
    use crate::{Field, Primitive};

    pub fn types(specs: Vec<step1::TypeSpecification>) -> Vec<Type> {
        specs
            .into_iter()
            .map(|spec| {
                let TypeSpecification { h4: name, p: descr, table } = spec;
                let fields = table.into_iter().map(|mut row| {
                    let (name, ty, mut descr) = (row.remove(0), row.remove(0), row.remove(0));
                    let ty = parse_type(&name, &ty, &mut descr);
                    Field { name, ty, descr }
                })
                .collect::<Vec<_>>();

                Type {
                    name,
                    descr: descr.unwrap_or(String::from("")),
                    fields,
                }
            })
            .collect()
    }

    fn parse_type(name: &str, ty: &str, descr: &mut String) -> Primitive {
        if descr.starts_with("Optional. ") {
            *descr = String::from(&descr[10..]);
            return Primitive::Option(Box::new(parse_type(name, ty, descr)))
        }

        // TODO: I64
        if ty.starts_with(" ") {
            // very stupid way to remove spaces, but ok
            return parse_type(name, &ty[1..], descr)
        }

        if ty.starts_with("Array of ") {
            return Primitive::Array(Box::new(parse_type(name, &ty[8..], descr)))
        }

        match ty {
            // Dirty hack, but it works, ok?
            "Integer" => if descr.contains("64") {
                Primitive::I64
            } else {
                Primitive::I32
            },
            "String" => match name {
                "parse_mode" => Primitive::ParseMode,
                _ => Primitive::String,
            },
            "Boolean" => Primitive::Bool,
            "True" => Primitive::True,
            "Integer or String" => Primitive::ChatId,
            "Float" => Primitive::F32,
            "InputFile or String" => Primitive::InputFile,
            s => Primitive::Struct { name: String::from(s) },
        }
    }
}