#[derive(Debug, serde::Deserialize)]
pub struct MobPushConfig {
    pub key: String,
    pub secret: String,
}

pub trait MobPushConfigTrait {
    fn get_key(&self) -> &str;
    fn get_secret(&self) -> &str;
}
