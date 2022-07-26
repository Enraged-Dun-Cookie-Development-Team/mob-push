#[derive(Debug, serde::Deserialize)]
pub struct MobPushConfig {
    pub key: String,
    pub secret: String,
}
