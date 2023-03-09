use std::borrow::Cow;

use serde::{ser::SerializeStruct, Serialize};

use crate::{
    config::get_config,
    push_notify::{
        android::AndroidNotify, ios::IosNotify, Notify, NotifySerialize, SerializeInformation,
    },
    user_subscribe::UserMobId,
    PushEntity, PushForward,
};

pub struct PushTarget {
    pub target_user: Vec<String>,
}

impl PushTarget {
    pub fn new(user_iter: &mut impl Iterator<Item = impl UserMobId>) -> Option<Self> {
        let mut vec = Vec::with_capacity(1000);
        for _ in 0..1000 {
            if let Some(user) = user_iter.next() {
                let user = user.get_mob_id().to_string();
                vec.push(user);
            } else {
                break;
            }
        }

        if vec.is_empty() {
            None
        } else {
            Some(Self { target_user: vec })
        }
    }
}

impl Serialize for PushTarget {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut push_target = serializer.serialize_struct("pushTarget", 2)?;

        push_target.serialize_field("target", &4)?;
        push_target.serialize_field("rids", &self.target_user)?;

        push_target.end()
    }
}

pub struct Forward(PushForward);

impl Forward {
    pub fn new<T: PushEntity>(data: &T) -> Self {
        let mut push_forward = PushForward::HomePage;
        data.push_forward(&mut push_forward);

        Self(push_forward)
    }
}

impl Serialize for Forward {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("pushForward", self.0.serialize_field())?;
        NotifySerialize::serialize::<S>(&self.0, &mut s)?;
        s.end()
    }
}

pub struct PushNotify<'p, A = Notify<AndroidNotify>, I = Notify<IosNotify>>
where
    A: Serialize + 'static,
    I: Serialize + 'static,
{
    body: &'p str,
    title: Cow<'p, str>,
    android_notify: A,
    ios_notify: I,
}

impl<'p> PushNotify<'p> {
    pub fn new_with_builder<T: PushEntity>(data: &'p T) -> Self {
        let mut android_notify = AndroidNotify::default().into_notify();
        data.android_notify(&mut android_notify);
        let mut ios_notify = IosNotify::default().into_notify();
        data.ios_notify(&mut ios_notify);

        Self {
            body: data.get_send_content().as_ref(),
            title: data.get_title(),
            android_notify,
            ios_notify,
        }
    }
}

impl<'p> Serialize for PushNotify<'p> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut len = 4;
        if self.android_notify.need_serialize() {
            len += 1;
        }
        if self.ios_notify.need_serialize() {
            len += 1;
        }

        let mut notify = serializer.serialize_struct("PushNotify", len)?;

        notify.serialize_field("plats", &[1, 2])?;
        notify.serialize_field("content", &self.body)?;
        notify.serialize_field("type", &1)?;
        notify.serialize_field("title", &self.title)?;

        if self.android_notify.need_serialize() {
            notify.serialize_field("androidNotify", &self.android_notify)?;
        }
        if self.ios_notify.need_serialize() {
            notify.serialize_field("iosNotify", &self.ios_notify)?;
        }

        notify.end()
    }
}

pub(crate) struct CreatePush<'p> {
    pub push_target: PushTarget,
    pub push_notify: PushNotify<'p>,
    pub push_forward: Forward,
}

impl<'p> Serialize for CreatePush<'p> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut push_body = serializer.serialize_struct("CreatePush", 4)?;

        push_body.serialize_field("source", &"webapi")?;
        push_body.serialize_field("appkey", &get_config().key)?;
        push_body.serialize_field("pushTarget", &self.push_target)?;
        push_body.serialize_field("pushNotify", &self.push_notify)?;
        push_body.serialize_field("pushForward", &self.push_forward)?;

        push_body.end()
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct Respond {
    pub(crate) status: u16,
    #[serde(rename = "res")]
    pub(crate) _res: Option<ResBody>,
    pub(crate) error: Option<String>,
}
#[derive(Debug, serde::Deserialize)]
pub(crate) struct ResBody {
    #[serde(rename = "batchId")]
    pub(crate) _batch_id: String,
}

#[cfg(test)]
mod test_serde {
    use std::borrow::Cow;

    use crate::{
        config::load_from_test,
        push_notify::{android::AndroidNotify, ios::IosNotify, SerializeInformation},
    };

    use super::CreatePush;

    #[test]
    fn test_serde() {
        load_from_test();

        let c = CreatePush {
            push_target: super::PushTarget {
                target_user: vec!["abc".to_string(), "cdde".to_string()],
            },
            push_notify: super::PushNotify {
                body: &String::from(r#"{"aab":11}"#),
                android_notify: AndroidNotify::default().into_notify(),
                ios_notify: IosNotify::default().into_notify(),
                title: Cow::Borrowed("12345"),
            },
            push_forward: super::Forward(crate::PushForward::HomePage),
        };

        let string = serde_json::to_string_pretty(&c).unwrap();

        println!("{string}")
    }
}
