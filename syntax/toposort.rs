use crate::syntax::report::Errors;
use crate::syntax::{Api, Struct, Type, Types};
use std::collections::btree_map::{BTreeMap as Map, Entry};

enum Mark {
    Visiting,
    Visited,
}

pub fn sort<'a>(cx: &mut Errors, apis: &'a [Api], types: &Types<'a>) -> Vec<&'a Struct> {
    let mut sorted = Vec::new();
    let ref mut marks = Map::new();
    for api in apis {
        if let Api::Struct(strct) = api {
            visit(cx, strct, &mut sorted, marks, types);
        }
    }
    sorted
}

fn visit<'a>(
    cx: &mut Errors,
    strct: &'a Struct,
    sorted: &mut Vec<&'a Struct>,
    marks: &mut Map<*const Struct, Mark>,
    types: &Types<'a>,
) {
    match marks.entry(strct) {
        Entry::Occupied(entry) => match entry.get() {
            Mark::Visiting => panic!("not a DAG"), // FIXME
            Mark::Visited => return,
        },
        Entry::Vacant(entry) => {
            entry.insert(Mark::Visiting);
        }
    }
    for field in &strct.fields {
        if let Type::Ident(ident) = &field.ty {
            if let Some(inner) = types.structs.get(&ident.rust) {
                visit(cx, inner, sorted, marks, types);
            }
        }
    }
    marks.insert(strct, Mark::Visited);
    sorted.push(strct);
}
