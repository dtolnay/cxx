use serde::Deserialize;

pub type Node = clang_ast::Node<Clang>;

#[derive(Deserialize)]
pub enum Clang {
    NamespaceDecl(NamespaceDecl),
    EnumDecl(EnumDecl),
    EnumConstantDecl(EnumConstantDecl),
    ImplicitCastExpr,
    ConstantExpr(ConstantExpr),
    Unknown,
}

#[derive(Deserialize)]
pub struct NamespaceDecl {
    pub name: Option<Box<str>>,
}

#[derive(Deserialize)]
pub struct EnumDecl {
    pub name: Option<Box<str>>,
    #[serde(rename = "fixedUnderlyingType")]
    pub fixed_underlying_type: Option<Type>,
}

#[derive(Deserialize)]
pub struct EnumConstantDecl {
    pub name: Box<str>,
}

#[derive(Deserialize)]
pub struct ConstantExpr {
    pub value: Box<str>,
}

#[derive(Deserialize)]
pub struct Type {
    #[serde(rename = "qualType")]
    pub qual_type: Box<str>,
    #[serde(rename = "desugaredQualType")]
    pub desugared_qual_type: Option<Box<str>>,
}

#[cfg(all(test, target_pointer_width = "64"))]
const _: [(); std::mem::size_of::<Node>()] = [(); 88];
