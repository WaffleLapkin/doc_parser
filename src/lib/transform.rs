use super::{parse, Type};
use crate::parse::TypeSpec;
use crate::{Field, Primitive, Change};

pub fn types(specs: Vec<parse::TypeSpec>) -> Vec<Type> {
    specs
        .into_iter()
        .map(|spec| {
            let TypeSpec { h4: name, p: descr, table } = spec;
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

pub fn recent_changes(specs: Vec<parse::Change>) -> Vec<Change> {
    specs
        .into_iter()
        .map(|parse::Change { h4, p, ul }| Change {
            date: h4,
            version: p.unwrap_or(String::from("")),
            changes: ul,
        })
        .collect::<Vec<_>>()
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
