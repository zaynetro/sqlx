use std::fmt::Display;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

pub use input::QueryMacroInput;
pub use query::expand_query;

use std::env;
use std::path::Path;

use crate::database::DatabaseExt;

use crate::query_macros::data::QueryData;
use sqlx::connection::Connection;
use sqlx::database::Database;

mod args;
mod data;
mod input;
mod output;
mod query;

pub fn expand_input(mut input: QueryMacroInput) -> crate::Result<TokenStream> {
    // file source is read to a string here
    let source = input.src.resolve(input.src_span)?;

    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").map_err(|| "`CARGO_MANIFEST_DIR` must be set")?;

    // If a .env file exists at CARGO_MANIFEST_DIR, load environment variables from this,
    // otherwise fallback to default dotenv behaviour.
    let env_path = Path::new(&manifest_dir).join(".env");
    if env_path.exists() {
        dotenv::from_path(&env_path)
            .map_err(|e| format!("failed to load environment from {:?}, {}", env_path, e))?
    }

    // if `dotenv` wasn't initialized by the above we make sure to do it here
    let query_data = match dotenv::var("DATABASE_URL").ok() {
        Some(db_url) => QueryData::from_db(&db_url, source)?,
        #[cfg(feature = "offline")]
        None => {
            let data_file_path = Path::new(&manifest_dir).join("sqlx-data.json");

            if data_file_path.exists() {
                QueryData::from_data_file(data_file_path, source)
            } else {
                return Err(
                    "`DATABASE_URL` must be set, or `cargo sqlx prepare` must have been run \
                     and sqlx-data.json must exist, to use query macros"
                        .into(),
                );
            }
        }
        #[cfg(not(feature = "offline"))]
        None => return Err("`DATABASE_URL` must be set to use query macros".into()),
    };

    // validate at the minimum that our args match the query's input parameters
    if input.arg_names.len() != query_data.input_types.len() {
        return Err(syn::Error::new(
            Span::call_site(),
            format!(
                "expected {} parameters, got {}",
                query_data.param_types.len(),
                input.arg_names.len()
            ),
        )
        .into());
    }

    #[cfg(feature = "offline")]
    {
        let target_dir = env::var("TARGET_DIR")
            .map_or_else(
                || Path::new(&manifest_dir).join("target"),
                std::path::PathBuf::from,
            )
            .join("sqlx");

        query_data.save_in(target_dir, input.src_span)?;
    }

    Ok()
}

pub async fn expand_query_file<C: Connection>(
    input: QueryMacroInput,
    conn: C,
    checked: bool,
) -> crate::Result<TokenStream>
where
    C::Database: DatabaseExt + Sized,
    <C::Database as Database>::TypeInfo: Display,
{
    expand_query(input.expand_file_src().await?, conn, checked).await
}

pub async fn expand_query_as<C: Connection>(
    input: QueryAsMacroInput,
    mut conn: C,
    checked: bool,
) -> crate::Result<TokenStream>
where
    C::Database: DatabaseExt + Sized,
    <C::Database as Database>::TypeInfo: Display,
{
    let describe = input.query_input.describe_validate(&mut conn).await?;

    if describe.result_columns.is_empty() {
        return Err(syn::Error::new(
            input.query_input.source_span,
            "query must output at least one column",
        )
        .into());
    }

    let args_tokens = args::quote_args(&input.query_input, &describe, checked)?;

    let query_args = format_ident!("query_args");

    let columns = output::columns_to_rust(&describe)?;
    let output = output::quote_query_as::<C::Database>(
        &input.query_input.source,
        &input.as_ty.path,
        &query_args,
        &columns,
        checked,
    );

    let arg_names = &input.query_input.arg_names;

    Ok(quote! {
        macro_rules! macro_result {
            (#($#arg_names:expr),*) => {{
                use sqlx::arguments::Arguments as _;

                #args_tokens

                #output
            }}
        }
    })
}

pub async fn expand_query_file_as<C: Connection>(
    input: QueryAsMacroInput,
    conn: C,
    checked: bool,
) -> crate::Result<TokenStream>
where
    C::Database: DatabaseExt + Sized,
    <C::Database as Database>::TypeInfo: Display,
{
    expand_query_as(input.expand_file_src().await?, conn, checked).await
}
