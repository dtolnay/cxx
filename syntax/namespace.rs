use crate::syntax::qualified::QualifiedName;
use quote::IdentFragment;
use std::fmt::{self, Display};
use std::slice::Iter;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{Attribute, Expr, Ident, Lit, Meta, Token};

mod kw {
    syn::custom_keyword!(namespace);
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct Namespace {
    segments: Vec<Ident>,
}

impl Namespace {
    pub(crate) const ROOT: Self = Namespace {
        segments: Vec::new(),
    };

    pub(crate) fn iter(&self) -> Iter<Ident> {
        self.segments.iter()
    }

    /// Parses `namespace = ...` (if present) from `attr`.
    /// Typically `attr` represents `#[cxx::bridge(...)]` attribute.
    #[allow(dead_code)] // Only used from tests and cxx-build, but not from cxxbridge-macro
    pub(crate) fn parse_attr(attr: &Attribute) -> Result<Namespace> {
        if let Meta::Path(_) = attr.meta {
            Ok(Namespace::ROOT)
        } else {
            attr.parse_args_with(Namespace::parse_stream)
        }
    }

    /// Parses `namespace = ...` (if present) from `input`.
    /// Typically `inputs` represents the "body" of a `#[cxx::bridge(...)]` attribute.
    pub(crate) fn parse_stream(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Namespace::ROOT);
        }

        input.parse::<kw::namespace>()?;
        input.parse::<Token![=]>()?;
        let namespace = input.parse::<Namespace>()?;
        input.parse::<Option<Token![,]>>()?;
        Ok(namespace)
    }

    pub(crate) fn parse_meta(meta: &Meta) -> Result<Self> {
        if let Meta::NameValue(meta) = meta {
            match &meta.value {
                Expr::Lit(expr) => {
                    if let Lit::Str(lit) = &expr.lit {
                        let segments = QualifiedName::parse_quoted(lit)?.segments;
                        return Ok(Namespace { segments });
                    }
                }
                Expr::Path(expr)
                    if expr.qself.is_none()
                        && expr
                            .path
                            .segments
                            .iter()
                            .all(|segment| segment.arguments.is_none()) =>
                {
                    let segments = expr
                        .path
                        .segments
                        .iter()
                        .map(|segment| segment.ident.clone())
                        .collect();
                    return Ok(Namespace { segments });
                }
                _ => {}
            }
        }
        Err(Error::new_spanned(meta, "unsupported namespace attribute"))
    }
}

impl Default for &Namespace {
    fn default() -> Self {
        const ROOT: &Namespace = &Namespace::ROOT;
        ROOT
    }
}

impl Parse for Namespace {
    fn parse(input: ParseStream) -> Result<Self> {
        let segments = QualifiedName::parse_quoted_or_unquoted(input)?.segments;
        Ok(Namespace { segments })
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for segment in self {
            write!(f, "{}$", segment)?;
        }
        Ok(())
    }
}

impl IdentFragment for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<'a> IntoIterator for &'a Namespace {
    type Item = &'a Ident;
    type IntoIter = Iter<'a, Ident>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> FromIterator<&'a Ident> for Namespace {
    fn from_iter<I>(idents: I) -> Self
    where
        I: IntoIterator<Item = &'a Ident>,
    {
        let segments = idents.into_iter().cloned().collect();
        Namespace { segments }
    }
}

#[cfg(test)]
mod test {
    use crate::syntax::test_support::parse_apis;
    use crate::syntax::Api;
    use quote::quote;

    #[test]
    fn test_top_level_namespace() {
        let apis = parse_apis(quote! {
            #[cxx::bridge(namespace = "top_level_namespace")]
            mod ffi {
                unsafe extern "C++" {
                    fn foo();
                }
            }
        })
        .unwrap();
        let [Api::CxxFunction(f)] = &apis[..] else {
            panic!("Got unexpected apis");
        };
        assert_eq!("top_level_namespace$foo", f.name.to_symbol().to_string());
    }
}
