use std::{
    convert::Infallible,
    fmt::Debug,
    marker::{PhantomData, Send, Sync},
    time::Duration,
};

use mob_push::{
    self, load_config_from_default, BoxResultFuture, MobPusher, PushEntity, SubscribeFilter,
    UserMobId, UserSubscribeManage,
};
use serde::{ser::SerializeStruct, Serialize};
use tokio::time;

#[derive(Debug)]
struct TestMsg<A = (), I = ()> {
    id: String,
    android: A,
    ios: I,
}

impl Default for TestMsg {
    fn default() -> Self {
        Self {
            id: String::from("test-1"),
            android: (),
            ios: (),
        }
    }
}
impl<A, I> TestMsg<A, I> {
    fn set_android<AN>(self, android: AN) -> TestMsg<AN, I> {
        TestMsg {
            id: self.id,
            android,
            ios: self.ios,
        }
    }
}

impl<A, I> Serialize for TestMsg<A, I> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("小刻食堂测试信息"))
    }
}

impl<A, I> PushEntity for TestMsg<A, I>
where
    A: Serialize + 'static + Send + Sync,
    I: Serialize + 'static + Send + Sync,
{
    type Resource = i32;

    fn get_resource(&self) -> &Self::Resource {
        &11
    }

    type AndroidNotify = A;

    fn get_android_notify(&self) -> &Self::AndroidNotify {
        &self.android
    }

    type IosNotify = I;

    fn get_ios_notify(&self) -> &Self::IosNotify {
        &self.ios
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

struct Manage<A = (), I = ()> {
    _p: PhantomData<(A, I)>,
}

impl<A, I> Manage<A, I> {
    fn new() -> Self {
        Self { _p: PhantomData }
    }
}

impl<A, I> UserSubscribeManage for Manage<A, I>
where
    A: Serialize + 'static + Send + Sync,
    I: Serialize + 'static + Send + Sync,
{
    type UserIdentify = User;

    type PushData = TestMsg<A, I>;

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
            vec![
                User {
                    mob_id: "65l05lvwtep0fls".into(),
                },
                User {
                    mob_id: "65kzw5w9iulerk0".into(),
                },
            ]
        } else {
            vec![]
        };

        Box::pin(async move { Ok(resp) })
    }
}

fn test_pushing<A, I, F>(msg: F)
where
    A: Serialize + Sync + Send + 'static + Debug,
    I: Serialize + Send + Sync + 'static + Debug,
    F: FnOnce() -> TestMsg<A, I>,
{
    load_config_from_default();
    let (mob_push, sender, mut err_rx) = MobPusher::new(Manage::<A, I>::new(), 8);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Rt start Error");

    rt.block_on(async move {
        let handle = tokio::spawn(mob_push.start_up());

        sender.send(msg()).await.unwrap();

        time::sleep(Duration::from_secs(1)).await;

        drop(sender);

        let err = time::timeout(Duration::from_millis(500), err_rx.recv()).await;
        if let Ok(err) = err {
            println!("{err:?}")
        }

        handle.abort();
    })
}

#[test]
// #[ignore = "conflict action"]
fn test_push() {
    test_pushing(|| TestMsg::default());
}

#[derive(Debug)]
struct AndroidBadge;

impl Serialize for AndroidBadge {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut badge = serializer.serialize_struct("androidNotify", 2)?;

        badge.serialize_field("androidBadgeType", &1)?;
        badge.serialize_field("androidBadge", &0)?;

        badge.end()
    }
}

/// 角标数值没啥意义
/// 每次推送都是+1
#[test]
fn test_android_badge() {
    test_pushing(|| TestMsg::default().set_android(AndroidBadge));
}

#[derive(Debug)]
struct AndroidWarn;

impl Serialize for AndroidWarn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut warn = serializer.serialize_struct("androidNotify", 1)?;
        warn.serialize_field("warn", "12")?;

        warn.end()
    }
}

#[test]
fn test_android_warn() {
    test_pushing(|| TestMsg::default().set_android(AndroidWarn));
}

/// 使用image 推送可行
#[derive(Debug)]
struct AndroidIcon;

impl Serialize for AndroidIcon {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let icon = "https://static.mob.com/www_mob_com/.nuxt/dist/client/img/62893e6.png";
        let mut icon_only = serializer.serialize_struct("androidNotify", 1)?;
        icon_only.serialize_field("image", icon)?;

        icon_only.end()
    }
}

#[test]
fn test_icon() {
    test_pushing(|| TestMsg::default().set_android(AndroidIcon));
}

/// - 长内容
///
/// 如果原来有就附加
/// 只会推送多行信息
///
/// - 大图
///
/// 没有成功推送过
///
/// - 横幅
///
/// 没有成功推送过
///
///

#[derive(Debug)]
struct AndroidStyle;

impl Serialize for AndroidStyle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut style = serializer.serialize_struct("androidNotify", 2)?;

        style.serialize_field(
            "content",
            &[
                "555555555333333333333333333333333333333333333333333333",
                "你好",
            ],
        )?;
        style.serialize_field("style", &1)?;

        style.end()
    }
}

#[test]
fn test_style() {
    test_pushing(|| TestMsg::default().set_android(AndroidStyle));
}

#[derive(Debug, serde::Serialize)]
struct Wrapper {
    #[serde(rename = "customStyle")]
    custom_style: CustomStyle,
}

/// 用户定义的样式
/// 不知道啥用
#[derive(Debug)]
struct CustomStyle;

impl Serialize for CustomStyle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut style = serializer.serialize_struct("CustomStyle", 3)?;

        style.serialize_field("styleNo", &3)?;

        style.serialize_field("buttonCopy", "欸嘿")?;

        style.serialize_field("buttonJumpUrl", "https://www.bilibili.com/")?;

        style.end()
    }
}
#[test]
fn test_custom_style() {
    test_pushing(|| {
        TestMsg::default().set_android(Wrapper {
            custom_style: CustomStyle,
        })
    });
}
