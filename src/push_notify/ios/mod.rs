use serde::{ser::SerializeStruct, Serialize};

pub use self::{
    apn::{ApnPush, IosPushSound},
    badge::{IosBadge, IosBadgeType},
    rich_text::{IosRichText, IosRichTextType},
};

mod apn;
mod badge;
mod rich_text;

pub trait IosNotify: IosRichText + ApnPush + IosBadge {
    fn subtitle(&self) -> Option<String> {
        None
    }

    fn content_available(&self) -> Option<()> {
        None
    }
}

pub(crate) struct IosNotifyWrapper<'ios, N: IosNotify> {
    inner: &'ios N,
}

impl<'ios, N: IosNotify> IosNotifyWrapper<'ios, N> {
    fn need_felid(&self) -> usize {
        let mut init = 0;
        // badge
        if self.inner.get_badge().is_some() {
            init += 2;
        }
        // apn
        if self.inner.category().is_some() {
            init += 1;
        }

        if self.inner.sound().is_some() {
            init += 1;
        }

        if self.inner.rich_text().is_some() {
            init += 3;
        }

        if self.inner.subtitle().is_some() {
            init += 1;
        }

        if self.inner.content_available().is_some() {
            init += 1;
        }

        init
    }

    #[allow(dead_code)]
    pub(super) fn need_serialize(&self) -> bool {
        self.need_felid() > 0
    }
}

impl<'ios, N: IosNotify> Serialize for IosNotifyWrapper<'ios, N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut ios_notify = serializer.serialize_struct("IosNotify", self.need_felid())?;

        // badge
        if let Some(badge) = self.inner.get_badge() {
            match badge {
                IosBadgeType::Abs(num) => {
                    ios_notify.serialize_field("badge", &num)?;
                    ios_notify.serialize_field("badgeType", &1)?;
                }
                IosBadgeType::Adding(value) => {
                    ios_notify.serialize_field("badge", &value)?;
                    ios_notify.serialize_field("badgeType", &2)?;
                }
            }
        }

        // apn
        if let Some(category) = self.inner.category() {
            ios_notify.serialize_field("category", &category)?;
        }
        if let Some(sound) = self.inner.sound() {
            ios_notify.serialize_field("sound", &sound)?;
        }

        // sub title
        if let Some(sub_title) = self.inner.subtitle() {
            ios_notify.serialize_field("subtitle", &sub_title)?;
        }

        // content available
        if self.inner.content_available().is_some() {
            ios_notify.serialize_field("contentAvailable", &1)?;
        }

        if let Some(rich_text) = self.inner.rich_text() {
            ios_notify.serialize_field("mutableContent", &1)?;

            match rich_text {
                IosRichTextType::None => {}
                IosRichTextType::Picture(text) => {
                    ios_notify.serialize_field("attachmentType", &1)?;
                    ios_notify.serialize_field("attachment", &text)?;
                }
                IosRichTextType::Video(v) => {
                    ios_notify.serialize_field("attachmentType", &2)?;
                    ios_notify.serialize_field("attachment", &v)?;
                }
                IosRichTextType::Voice(v) => {
                    ios_notify.serialize_field("attachmentType", &3)?;
                    ios_notify.serialize_field("attachment", &v)?;
                }
            }
        }

        ios_notify.end()
    }
}

#[cfg(test)]
mod test {

    use super::{
        apn::{ApnPush, IosPushSound},
        badge::{IosBadge, IosBadgeType},
        rich_text::IosRichText,
        IosNotify, IosNotifyWrapper,
    };

    struct Test;

    impl IosNotify for Test {
        fn subtitle(&self) -> Option<String> {
            Some("Test Sub Title".into())
        }

        fn content_available(&self) -> Option<()> {
            Some(())
        }
    }

    impl IosBadge for Test {
        fn get_badge(&self) -> Option<IosBadgeType> {
            Some(IosBadgeType::Adding(12))
        }
    }

    impl ApnPush for Test {
        fn sound(&self) -> Option<IosPushSound> {
            Some(IosPushSound::Custom("123456".into()))
        }
    }

    impl IosRichText for Test {}

    #[test]
    fn test() {
        let wrap = IosNotifyWrapper { inner: &Test };

        let out = serde_json::to_string_pretty(&wrap).unwrap();

        println!("{out}")
    }
}
