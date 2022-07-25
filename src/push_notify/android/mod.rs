pub mod notify_style;
use serde::{ser::SerializeStruct, Serialize};
use typed_builder::TypedBuilder;

pub use self::{
    badge::Badge,
    image::Image,
    notify_style::NotifyStyle,
    sound::{Sound, Warn},
};

use super::NotifySerialize;

pub mod badge;
pub mod image;
pub mod sound;

#[derive(Debug, TypedBuilder, Default)]
pub struct AndroidNotify {
    #[builder(default, setter(strip_option))]
    notify_style: Option<NotifyStyle>,
    #[builder(default, setter(strip_option))]
    badge: Option<Badge>,
    #[builder(default, setter(strip_option))]
    image: Option<Image>,
    #[builder(default, setter(strip_option))]
    sound: Option<Sound>,
    #[builder(default, setter(strip_option))]
    warn: Option<Warn>,
}

impl AndroidNotify {
    pub fn set_notify_style(&mut self, style: NotifyStyle) -> &mut Self {
        self.notify_style.replace(style);
        self
    }
    pub fn set_badge(&mut self, badge: Badge) -> &mut Self {
        self.badge.replace(badge);
        self
    }
    pub fn set_image(&mut self, image: Image) -> &mut Self {
        self.image.replace(image);
        self
    }
    pub fn set_sound(&mut self, sound: Sound) -> &mut Self {
        self.sound.replace(sound);
        self
    }
    pub fn set_warn(&mut self, warn: Warn) -> &mut Self {
        self.warn.replace(warn);
        self
    }
}

impl Serialize for AndroidNotify {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let fields = self.notify_style.serialize_field()
            + self.badge.serialize_field()
            + self.image.serialize_field()
            + self.sound.serialize_field()
            + self.warn.serialize_field();

        let mut serialize_struct = serializer.serialize_struct("androidNotify", fields)?;

        self.notify_style.serialize::<S>(&mut serialize_struct)?;
        self.badge.serialize::<S>(&mut serialize_struct)?;
        self.image.serialize::<S>(&mut serialize_struct)?;
        self.sound.serialize::<S>(&mut serialize_struct)?;
        self.warn.serialize::<S>(&mut serialize_struct)?;

        serialize_struct.end()
    }
}

#[cfg(test)]
mod test {
    use super::{
        badge::Badge,
        notify_style::{CustomStyle, NotifyStyle, StyleId},
        sound::WarnSound,
        AndroidNotify,
    };

    #[test]
    fn test_builder() {
        let notify = AndroidNotify::builder()
        // 设置推送消息style
        .notify_style(NotifyStyle::new_big_vision("https://i0.hdslb.com/bfs/archive/94bdaa89d9e1775f04bdfb705512a61e5de70628.jpg@672w_378h_1c"))
        // 设置推送角标
        .badge(Badge::new_add(1))
        // 设置推送声音
        .sound("114514".into())
        // 设置推送提示音
        .warn(WarnSound::Prompt & WarnSound::IndicatorLight & WarnSound::Vibration)
        .build();

        let string = serde_json::to_string_pretty(&notify).unwrap();

        println!("{string}")
    }

    #[test]
    fn test_modify() {
        let mut notify = AndroidNotify::default();
        notify
            .set_notify_style(NotifyStyle::Custom(
                CustomStyle::builder().style(StyleId::One).build(),
            ))
            .set_badge(Badge::Add(1))
            .set_warn(WarnSound::Prompt & WarnSound::IndicatorLight);

        let string = serde_json::to_string_pretty(&notify).unwrap();

        println!("{string}")
    }
}
