use crate::syntax::namespace::Namespace;
use crate::syntax::ExternFn;

pub fn extern_fn(namespace: &Namespace, efn: &ExternFn) -> String {
    let receiver_type = match &efn.receiver {
        Some(receiver) => receiver.ident.to_string(),
        None => "_".to_string(),
    };
    format!("{}cxxbridge02${}${}", namespace, receiver_type, efn.ident)
}
