use std::{convert::Infallible, time::Duration};

use mob_push::{
    self, load_config_from_default, BoxResultFuture, MobPusher, PushEntity, SubscribeFilter,
    UserMobId, UserSubscribeManage,
};
use serde::{ser::SerializeStruct, Serialize};
use tokio::time;

#[derive(Debug)]
struct TestMsg {
    id: String,
}

impl Default for TestMsg {
    fn default() -> Self {
        Self {
            id: String::from("test-1"),
        }
    }
}

impl Serialize for TestMsg {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut msg = serializer.serialize_struct("TestMsg", 3)?;

        msg.serialize_field("base", "测试用的消息")?;
        msg.serialize_field("msg", "测试消息内容")?;
        msg.serialize_field("id", &self.id)?;

        msg.end()
    }
}

impl PushEntity for TestMsg {
    type Resource = i32;

    fn get_resource(&self) -> &Self::Resource {
        &11
    }

    type Identity = String;

    fn get_identity(&self) -> &Self::Identity {
        &self.id
    }
}

struct User {
    mob_id: String,
}

impl UserMobId for User {
    type MobId = String;

    fn get_mob_id(&self) -> Self::MobId {
        self.mob_id.clone()
    }
}

struct Filter;

impl SubscribeFilter for Filter {
    type Data = TestMsg;

    type Err = Infallible;

    fn filter(input: impl Iterator<Item = Self::Data>) -> Result<Vec<Self::Data>, Self::Err> {
        Ok(input
            .filter(|data| data.get_resource() == &11 || data.get_resource() == &15)
            .collect())
    }

    fn contains(target: &<Self::Data as PushEntity>::Resource) -> Result<bool, Self::Err> {
        Ok(target == &11 || target == &15)
    }
}

struct Manage;

#[cfg(test)]
impl UserSubscribeManage for Manage {
    type UserIdentify = User;

    type PushData = TestMsg;

    type Filter = Filter;

    type Err = Infallible;

    fn fetch_subscribe_filter(
        &self,
        _user_id: &Self::UserIdentify,
    ) -> BoxResultFuture<Self::Filter, Self::Err> {
        Box::pin(async { Ok(Filter) })
    }

    fn check_subscribed(
        &self,
        _user_id: &Self::UserIdentify,
        data_resource: &<Self::PushData as PushEntity>::Resource,
    ) -> BoxResultFuture<bool, Self::Err> {
        let resp = Filter::contains(data_resource);
        Box::pin(async move { resp })
    }

    fn fetch_all_subscriber(
        &self,
        data_resource: &<Self::PushData as PushEntity>::Resource,
    ) -> BoxResultFuture<Vec<Self::UserIdentify>, Self::Err> {
        let resp = if data_resource == &11 || data_resource == &15 {
            vec![User {
                mob_id: "65kzw5w9iulerk0".into(),
            }]
        } else {
            vec![]
        };

        Box::pin(async move { Ok(resp) })
    }
}

#[tokio::test]
async fn test_push() {
    load_config_from_default();
    let (mob_push, sender, mut err_rx) = MobPusher::new(Manage, 8);

    let handle = tokio::spawn(mob_push.start_up());

    sender.send(TestMsg::default()).await.unwrap();

    time::sleep(Duration::from_secs(1)).await;

    drop(sender);

    let err = time::timeout(Duration::from_millis(500), err_rx.recv()).await;
    if let Ok(err) = err {
        println!("{err:?}")
    }

    handle.abort();
}
