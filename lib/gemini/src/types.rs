mod request;
mod response;

pub use request::*;
pub use response::*;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Content {
    pub parts: Option<Vec<Part>>,
    pub role: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Part {
    pub text: Option<String>,
}
