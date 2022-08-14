pub mod store;

use crate::store::{Error as StoreError, InternalError};
use sqlite::Error;

// type _Error = InternalError<Error>;
pub type Result<T> = std::result::Result<T, InternalError<Error>>;

impl From<Error> for InternalError<Error> {
    fn from(err: Error) -> Self {
        InternalError::Unknow(StoreError {
            message: String::from("unKnow"),
            code: 0,
            err: Some(err),
        })
    }
}
