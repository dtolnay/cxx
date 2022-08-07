use crate::syntax::cfg::CfgExpr;
use crate::syntax::namespace::Namespace;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{
    braced, token, Abi, Attribute, ForeignItem as RustForeignItem, ForeignItemFn, ForeignItemMacro,
    ForeignItemType, Ident, Item as RustItem, ItemEnum, ItemImpl, ItemStruct, ItemUse, LitStr,
    Token, TypePath, Visibility,
};

pub struct Module {
    pub cfg: CfgExpr,
    pub namespace: Namespace,
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub unsafety: Option<Token![unsafe]>,
    pub mod_token: Token![mod],
    pub ident: Ident,
    pub brace_token: token::Brace,
    pub content: Vec<Item>,
}

pub enum Item {
    Struct(ItemStruct),
    Enum(ItemEnum),
    ForeignMod(ItemForeignMod),
    Use(ItemUse),
    Impl(ItemImpl),
    Other(RustItem),
}

pub struct ItemForeignMod {
    pub attrs: Vec<Attribute>,
    pub unsafety: Option<Token![unsafe]>,
    pub abi: Abi,
    pub brace_token: token::Brace,
    pub items: Vec<ForeignItem>,
}

pub enum ForeignItem {
    Type(ForeignItemType),
    Fn(ForeignItemFn),
    Macro(ForeignItemMacro),
    Verbatim(TokenStream),
    Impl(ForeignItemImpl),
    Other(RustForeignItem),
}

pub struct ForeignItemImpl {
    pub attrs: Vec<Attribute>,
    pub unsafety: Option<Token![unsafe]>,
    pub impl_token: Token![impl],
    pub self_ty: TypePath,
    pub brace_token: token::Brace,
    pub items: Vec<ForeignImplItem>,
}

pub enum ForeignImplItem {
    Fn(ForeignItemFn),
}

impl Parse for Module {
    fn parse(input: ParseStream) -> Result<Self> {
        let cfg = CfgExpr::Unconditional;
        let namespace = Namespace::ROOT;
        let mut attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let unsafety: Option<Token![unsafe]> = input.parse()?;
        let mod_token: Token![mod] = input.parse()?;
        let ident: Ident = input.parse()?;

        let semi: Option<Token![;]> = input.parse()?;
        if let Some(semi) = semi {
            let span = quote!(#vis #mod_token #semi);
            return Err(Error::new_spanned(
                span,
                "#[cxx::bridge] module must have inline contents",
            ));
        }

        let content;
        let brace_token = braced!(content in input);
        attrs.extend(content.call(Attribute::parse_inner)?);

        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }

        Ok(Module {
            cfg,
            namespace,
            attrs,
            vis,
            unsafety,
            mod_token,
            ident,
            brace_token,
            content: items,
        })
    }
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let ahead = input.fork();
        ahead.parse::<Option<Token![unsafe]>>()?;
        if ahead.parse::<Option<Token![extern]>>()?.is_some()
            && ahead.parse::<Option<LitStr>>().is_ok()
            && ahead.peek(token::Brace)
        {
            let unsafety = input.parse()?;
            let mut foreign_mod = ItemForeignMod::parse(input)?;
            foreign_mod.attrs.splice(..0, attrs);
            foreign_mod.unsafety = unsafety;
            return Ok(Item::ForeignMod(foreign_mod));
        };

        let item = input.parse()?;
        match item {
            RustItem::Struct(mut item) => {
                item.attrs.splice(..0, attrs);
                Ok(Item::Struct(item))
            }
            RustItem::Enum(mut item) => {
                item.attrs.splice(..0, attrs);
                Ok(Item::Enum(item))
            }
            RustItem::ForeignMod(item) => Err(syn::parse::Error::new(
                item.span(),
                "Reached generic ForeignMod code instead of custom ItemForeignMod, this is a bug",
            )),
            RustItem::Impl(mut item) => {
                item.attrs.splice(..0, attrs);
                Ok(Item::Impl(item))
            }
            RustItem::Use(mut item) => {
                item.attrs.splice(..0, attrs);
                Ok(Item::Use(item))
            }
            other => Ok(Item::Other(other)),
        }
    }
}

impl Parse for ItemForeignMod {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = input.call(Attribute::parse_outer)?;
        let abi: Abi = input.parse()?;

        let content;
        let brace_token = braced!(content in input);
        attrs.extend(content.call(Attribute::parse_inner)?);
        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }
        Ok(ItemForeignMod {
            attrs,
            unsafety: None,
            abi,
            brace_token,
            items,
        })
    }
}

impl Parse for ForeignItem {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![impl]) {
            return Ok(ForeignItem::Impl(ForeignItemImpl::parse(input)?));
        }
        Ok(match RustForeignItem::parse(input)? {
            RustForeignItem::Type(ty) => ForeignItem::Type(ty),
            RustForeignItem::Fn(f) => ForeignItem::Fn(f),
            RustForeignItem::Macro(m) => ForeignItem::Macro(m),
            RustForeignItem::Verbatim(t) => ForeignItem::Verbatim(t),
            i => ForeignItem::Other(i),
        })
    }
}

impl Parse for ForeignItemImpl {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = input.call(Attribute::parse_outer)?;
        let unsafety: Option<Token![unsafe]> = input.parse()?;
        let impl_token: Token![impl] = input.parse()?;
        let self_ty: TypePath = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        attrs.extend(content.call(Attribute::parse_inner)?);
        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }
        Ok(ForeignItemImpl {
            attrs,
            unsafety,
            impl_token,
            self_ty,
            brace_token,
            items,
        })
    }
}

impl Parse for ForeignImplItem {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ForeignImplItem::Fn(input.parse()?))
    }
}
