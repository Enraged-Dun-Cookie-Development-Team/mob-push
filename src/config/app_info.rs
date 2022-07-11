#[derive(Debug, serde::Deserialize)]
pub struct App {
    pub(crate) key: String,
    pub(crate) secret: String,
}
