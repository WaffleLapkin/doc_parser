use select::document::Document;
use select::predicate::Name;
use itertools::Itertools;


/// Raw type specification from docs.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TypeSpec {
    /// Name of the type
    pub h4: String,
    /// Description of the type
    pub p: Option<String>,
    /// Fields of the type
    pub table: Vec<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Change {
    // date
    pub h4: String,
    // ver
    pub p: Option<String>, // todo option => ""
    // changes
    pub ul: Vec<String>,
}

/// Parse all types from ["Available types"] section in [telegram docs]
/// into raw [TypeSpecification]
///
/// ["Available types"]: https://core.telegram.org/bots/api#available-types
/// [telegram docs]: https://core.telegram.org/bots/api
/// [TypeSpecification]: self::TypeSpecification
pub fn parse_available_types(doc: &Document) -> Vec<TypeSpec> {
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
        .fold(Vec::<TypeSpec>::new(), |mut acc, node| {
            if node.is(Name("h4")) {
                // All <h4> in our interval is type names
                // So them we meet <h4>, it's new type definition
                acc.push(TypeSpec { h4: node.text(), p: None, table: vec![] });
            } else if node.is(Name("p")) {
                let last = acc.last_mut();
                let text = node.text();

                match last {
                    // Last type definition exists, but has no description
                    Some(TypeSpec { p: p @ None, .. }) => { *p = Some(text) },
                    // Last type definition exists, and has description
                    Some(TypeSpec { p: Some(t), .. }) => { t.push_str(&text) },
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

                let TypeSpec { table: v, .. } = last;
                v.extend(vec);
            }

            acc
        });

    res
}

pub fn parse_recent_changes(doc: Document) -> Vec<Change> {
    let start = doc
        .find(Name("h3"))
        .find(|n| n.text() == "Recent changes")
        .unwrap()
        .index();

    let stop = doc
        .find(Name("a"))
        .find(|n| n.text() == "See earlier changes »")
        .unwrap()
        .index();

    let res = (start..stop)
        .map(|i| doc.nth(i))
        .while_some()
        .fold(Vec::<Change>::new(), |mut acc, node| {
            if node.is(Name("h4")) {
                acc.push(Change { h4: node.text(), p: None, ul: vec![] });
            } else if node.is(Name("p")) && node.text().starts_with("Bot API") {
                let last = acc.last_mut();
                let text = node.text();

                match last {
                    // // Last type definition exists, but has no description
                    Some(Change { p: p @ None, .. }) => { *p = Some(text) },
                    // // Last type definition exists, and has description
                    Some(Change { p: Some(t), .. }) => { t.push_str(&text) },
                    // // Skip all <p> which are before first type definition
                    None => println!("Warn: skipped <p>: {}", node.text()),
                };
            } else if node.is(Name("ul")) {
                let vec = node
                    .children()
                    .filter(|tag| tag.is(Name("li")))
                    .map(|x| x.text())
                    .collect::<Vec<_>>();

                let last = acc.last_mut().expect("h4 before list");

                let Change { ul: v, .. } = last;
                v.extend(vec);
            }

            acc
        });

    res
}
