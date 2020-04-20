use crate::syntax::namespace::Namespace;
use crate::syntax::ExternFn;

pub fn extern_fn(namespace: &Namespace, efn: &ExternFn) -> String {
    let receiver = match &efn.receiver {
        Some(receiver) => receiver.ident.to_string() + "$",
        None => String::new(),
    };
    format!("{}cxxbridge02${}{}", namespace, receiver, efn.ident)
}
