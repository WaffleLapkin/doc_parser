#![allow(dead_code)]
//! Simple parser for [Telegram Bot API docs]
//!
//! [Telegram Bot API docs]: https://core.telegram.org/bots/api#available-methods

use select::document::Document;
use select::predicate::{Name};
use std::fs::File;
use std::any::type_name;

pub struct Structure {
    pub name: String,
    pub descr: String,
    pub fields: Vec<Field>,
}

pub struct Field {
    pub name: String,
    pub ty: Type,
    pub descr: String,
}

pub enum Type {
    I32, // tg's Integer
    I64, // tg's Integer in some cases (writers of the doc, I hate you)
    Str, // tg's String
    Bool, // tg's Boolean
    True, // tg's True
    Complex { name: String },
    Array(Box<Type>), // tg's Array of
    ChatId, // tg's Integer or String
    Option(Box<Type>), // tg's "Optional." in description
}

pub struct Method {
    pub name: String,
    pub descr: String,
    pub params: Vec<Param>,
    pub return_ty: Type,
}

pub struct Param {
    pub name: String,
    pub ty: Type,
    pub required: Required,
    pub descr: String,
}

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
    use super::{step1, Structure};
    use crate::step1::TypeSpecification;
    use crate::{Field, Type};

    pub fn types(specs: Vec<step1::TypeSpecification>) -> Vec<Structure> {
        specs
            .into_iter()
            .map(|spec| {
                let TypeSpecification { h4: name, p: descr, table } = spec;
                let fields = table.into_iter().map(|mut row| {
                    let (name, ty, descr) = (row.remove(0), row.remove(0), row.remove(0));
                    Field {
                        name,
                        ty: parse_type(&ty),
                        descr,
                    }
                })
                .collect::<Vec<_>>();

                Structure {
                    name,
                    descr: descr.unwrap_or(String::from("")),
                    fields,
                }
            })
            .collect()
    }

    fn parse_type(string: &str) -> Type {
//        I32, // tg's Integer
//        I64, // tg's Integer in some cases (writers of the doc, I hate you)
//        Str, // tg's String
//        Bool, // tg's Boolean
//        True, // tg's True
//        Complex(Structure),
//        Array(Box<Type>), // tg's Array of
//        ChatId, // tg's Integer or String
//        Option(Box<Type>), // tg's "Optional." in description

        // TODO: I64
        // TODO: Option
        if string.starts_with("Array of ") {
            return Type::Array(Box::new(parse_type(&string[8..])))
        }

        match string {
            "Integer" => Type::I32,
            "String" => Type::Str,
            "Boolean" => Type::Bool,
            "True" => Type::True,
            "Integer or String" => Type::ChatId,
            s => Type::Complex { name: String::from(s) },
        }
    }
}