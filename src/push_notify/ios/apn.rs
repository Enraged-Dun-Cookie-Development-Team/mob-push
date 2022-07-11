use serde::Serialize;

pub trait ApnPush {
    fn category(&self) -> Option<String> {
        None
    }
    fn sound(&self) -> Option<IosPushSound> {
        None
    }
}

pub enum IosPushSound {
    Default,
    None,
    Custom(String),
}

impl Serialize for IosPushSound {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            IosPushSound::Default => serializer.serialize_str("default"),
            IosPushSound::None => serializer.serialize_none(),
            IosPushSound::Custom(s) => serializer.serialize_str(&s),
        }
    }
}
