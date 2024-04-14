use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ApiError<'a> {
    pub error: &'a str,
    pub description: String,
}