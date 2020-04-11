use super::MsSql;
use crate::error::DatabaseError;
use crate::mssql::protocol::server::error::Error;
use std::error::Error as StdError;
use std::fmt::{self, Display};

#[derive(Debug)]
pub struct MsSqlError(pub(crate) Error);

impl DatabaseError for MsSqlError {
    fn message(&self) -> &str {
        &self.0.msg_text
    }

    fn as_ref_err(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }

    fn as_mut_err(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        self
    }

    fn into_box_err(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static> {
        self
    }
}

impl Display for MsSqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl StdError for MsSqlError {}

impl From<MsSqlError> for crate::Error {
    fn from(err: MsSqlError) -> Self {
        crate::Error::Database(Box::new(err))
    }
}
