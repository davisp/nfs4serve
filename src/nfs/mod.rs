pub mod api;
pub mod attrs;
mod connection;
pub mod constants;
mod ops;
mod status;
pub mod types;

pub use connection::{NfsConnection, NfsRequest};
pub use ops::NfsOperation;
pub use status::NfsStatus;

pub trait AsNfsStatus {
    fn as_status(&self) -> NfsStatus {
        NfsStatus::Ok
    }

    fn has_body(&self) -> bool {
        true
    }
}
