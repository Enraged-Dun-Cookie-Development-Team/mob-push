pub enum IosRichTextType {
    None,
    Picture(String),
    Video(String),
    Voice(String),
}

pub trait IosRichText {
    fn rich_text(&self) -> Option<IosRichTextType> {
        None
    }
}
