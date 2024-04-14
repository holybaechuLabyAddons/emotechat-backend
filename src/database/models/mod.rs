use thiserror::Error;

pub mod emote;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Error while interacting with the database: {0}")]
    Database(#[from] mongodb::error::Error),
}