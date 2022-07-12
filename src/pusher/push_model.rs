use std::any::TypeId;

use serde::{ser::SerializeStruct, Serialize};

use crate::{config::get_config, user_subscribe::UserMobId};

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

impl serde::Serialize for PushTarget {
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

pub struct PushNotify<'p, A = (), I = ()>
where
    A: Serialize + 'static,
    I: Serialize + 'static,
{
    body: String,
    android_notify: &'p A,
    ios_notify: &'p I,
}

impl<'p, A, I> PushNotify<'p, A, I>
where
    A: Serialize + 'static,
    I: Serialize + 'static,
{
    pub fn new<T: serde::Serialize>(
        body: &T,
        android: &'p A,
        ios: &'p I,
    ) -> Result<Self, serde_json::Error> {
        let body = serde_json::to_string(body)?;
        Ok(Self {
            body,
            android_notify: android,
            ios_notify: ios,
        })
    }
}

impl<'p, A, I> serde::Serialize for PushNotify<'p, A, I>
where
    A: Serialize + 'static,
    I: Serialize + 'static,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut len = if cfg!(debug_assertions) { 4 } else { 3 };
        if TypeId::of::<A>() != TypeId::of::<()>() {
            len += 1;
        }
        if TypeId::of::<I>() != TypeId::of::<()>() {
            len += 1;
        }

        let mut notify = serializer.serialize_struct("PushNotify", len)?;

        notify.serialize_field("plats", &[1, 2])?;
        if cfg!(debug_assertions) {
            notify.serialize_field("iosProduction", &0)?;
        }
        notify.serialize_field("content", &self.body)?;
        notify.serialize_field("type", &1)?;

        if TypeId::of::<A>() != TypeId::of::<()>() {
            notify.serialize_field("androidNotify", &self.android_notify)?;
        }
        if TypeId::of::<I>() != TypeId::of::<()>() {
            notify.serialize_field("iosNotify", &self.ios_notify)?;
        }

        notify.end()
    }
}

pub(crate) struct CreatePush<'p, A = (), I = ()>
where
    A: Serialize + 'static,
    I: Serialize + 'static,
{
    pub push_target: PushTarget,
    pub push_notify: PushNotify<'p, A, I>,
}

impl<'p, A, I> serde::Serialize for CreatePush<'p, A, I>
where
    A: Serialize + 'static,
    I: Serialize + 'static,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut push_body = serializer.serialize_struct("CreatePush", 4)?;

        push_body.serialize_field("source", &"webapi")?;
        push_body.serialize_field("appkey", &get_config().key)?;
        push_body.serialize_field("pushTarget", &self.push_target)?;
        push_body.serialize_field("pushNotify", &self.push_notify)?;

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
    use crate::config::load_from_test;

    use super::CreatePush;

    #[test]
    fn test_serde() {
        load_from_test();

        let c = CreatePush {
            push_target: super::PushTarget {
                target_user: vec!["abc".to_string(), "cdde".to_string()],
            },
            push_notify: super::PushNotify {
                body: String::from(r#"{"aab":11}"#),
                android_notify: &(),
                ios_notify: &(),
            },
        };

        let string = serde_json::to_string_pretty(&c).unwrap();

        println!("{string}")
    }
}
