use std::env;

use proc_macro2::{Ident, Span};
use quote::{format_ident, ToTokens};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Group;
use syn::{Error, Expr, ExprLit, ExprPath, Lit, LitBool, LitStr, Token};
use syn::{ExprArray, ExprGroup, Type};

use sqlx::connection::Connection;
use sqlx::describe::Describe;

use crate::runtime::fs;

/// Macro input shared by `query!()` and `query_file!()`
pub struct QueryMacroInput {
    pub(super) src: QuerySrc,
    pub(super) src_span: Span,

    pub(super) data_src: DataSrc,

    pub(super) record_type: RecordType,

    // `arg0 .. argN` for N arguments
    pub(super) arg_names: Vec<Ident>,
    pub(super) arg_exprs: Vec<Expr>,

    pub(super) unchecked_output: bool,
}

pub enum QuerySrc {
    String(String),
    File(String),
}

pub enum DataSrc {
    Env(String),
    DbUrl(String),
    File,
}

pub enum RecordType {
    Given(Type),
    Generated,
}

impl Parse for QueryMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut query_src: Option<(QuerySrc, Span)> = None;
        let mut data_src = DataSrc::Env("DATABASE_URL".into());
        let mut args: Option<Vec<Expr>> = None;
        let mut record_type = RecordType::Generated;
        let mut unchecked_output = false;

        let mut expect_comma = false;

        while !input.is_empty() {
            if expect_comma && !input.peek(Token![,]) {
                return Err(input.error("expected `,`"));
            }

            let key: Ident = input.parse()?;

            if input.peek(Token![=]) {
                return Err(input.error("expected `=`"));
            }

            if key == "source" {
                let lit_str = input.parse::<LitStr>()?;
                query_src = Some((QuerySrc::String(lit_str.value()), lit_str.span()));
            } else if key == "source_file" {
                let lit_str = input.parse::<LitStr>()?;
                query_src = Some((QuerySrc::File(lit_str.value()), lit_str.span()));
            } else if key == "args" {
                let exprs = input.parse::<ExprArray>()?;
                args = Some(exprs.elems.into_iter().collect())
            } else if key == "record_type" {
                record_type = RecordType::Given(input.parse()?);
            } else if key == "unchecked_output" {
                let lit_bool = input.parse::<LitBool>()?;
                unchecked_output = lit_bool.value;
            } else {
                return Err(syn::Error::new_spanned(key, "unexpected input key"));
            }

            expect_comma = true;
        }

        let (src, src_span) =
            query_src.ok_or_else(|| input.error("expected `source` or `source_file` key"))?;

        let arg_exprs = args.unwrap_or_default();
        let arg_names = (0..arg_exprs.len())
            .map(|i| format_ident!("arg{}", i))
            .collect();

        Ok(QueryMacroInput {
            src,
            src_span,
            data_src,
            record_type,
            arg_names,
            arg_exprs,
            unchecked_output,
        })
    }
}

impl QuerySrc {
    /// If the query source is a file, read it to a string. Otherwise return the query string.
    pub fn resolve(&mut self, source_span: Span) -> syn::Result<&str> {
        match self {
            QuerySrc::String(string) => Ok(string),
            QuerySrc::File(file) => {
                *self = QuerySrc::String(read_file_src(&file, source_span)?);
                self.resolve(source_span)
            }
        }
    }
}

async fn read_file_src(source: &str, source_span: Span) -> syn::Result<String> {
    use std::path::Path;

    let path = Path::new(source);

    if path.is_absolute() {
        return Err(syn::Error::new(
            source_span,
            "absolute paths will only work on the current machine",
        ));
    }

    // requires `proc_macro::SourceFile::path()` to be stable
    // https://github.com/rust-lang/rust/issues/54725
    if path.is_relative()
        && !path
            .parent()
            .map_or(false, |parent| !parent.as_os_str().is_empty())
    {
        return Err(syn::Error::new(
            source_span,
            "paths relative to the current file's directory are not currently supported",
        ));
    }

    let base_dir = env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        syn::Error::new(
            source_span,
            "CARGO_MANIFEST_DIR is not set; please use Cargo to build",
        )
    })?;

    let base_dir_path = Path::new(&base_dir);

    let file_path = base_dir_path.join(path);

    fs::read_to_string(&file_path).await.map_err(|e| {
        syn::Error::new(
            source_span,
            format!(
                "failed to read query file at {}: {}",
                file_path.display(),
                e
            ),
        )
    })
}
