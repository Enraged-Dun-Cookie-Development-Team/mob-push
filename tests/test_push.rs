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
    android: A,
    ios: I,
}

#[allow(clippy::derivable_impls)]
impl Default for TestMsg {
    fn default() -> Self {
        Self {
            android: (),
            ios: (),
        }
    }
}
impl<A, I> TestMsg<A, I> {
    fn set_android<AN>(self, android: AN) -> TestMsg<AN, I> {
        TestMsg {
            android,
            ios: self.ios,
        }
    }

    fn set_ios<IN>(self, ios: IN) -> TestMsg<A, IN> {
        TestMsg {
            android: self.android,
            ios,
        }
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

    type Content = str;

    fn get_send_content(&self) -> &Self::Content {
        "小刻食堂测试信息"
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

    fn filter(
        &self,
        input: impl Iterator<Item = Self::Data>,
    ) -> Result<Vec<Self::Data>, Self::Err> {
        Ok(input
            .filter(|data| data.get_resource() == &11 || data.get_resource() == &15)
            .collect())
    }

    fn contains(&self, target: &<Self::Data as PushEntity>::Resource) -> Result<bool, Self::Err> {
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
        let resp = Filter.contains(data_resource);
        Box::pin(async move { resp })
    }

    fn fetch_all_subscriber(
        &self,
        data_resource: &<Self::PushData as PushEntity>::Resource,
    ) -> BoxResultFuture<Vec<Self::UserIdentify>, Self::Err> {
        let resp = if data_resource == &11 || data_resource == &15 {
            vec![
                User {
                    mob_id: "65l063ct4qsghds".into(),
                },
                User {
                    mob_id: "65kzw5w9iulerk0".into(),
                },
                User {
                    mob_id: "65l05lvwtep0fls".into(),
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
fn test_push() {
    test_pushing(TestMsg::default);
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
        let _img = "https://static.mob.com/www_mob_com/.nuxt/dist/client/img/62893e6.png";
        let icon = "https://www.mob.com/favicon.ico";
        let mut icon_only = serializer.serialize_struct("androidNotify", 1)?;
        icon_only.serialize_field("image", icon)?;

        icon_only.end()
    }
}

#[test]
fn test_icon() {
    test_pushing(|| TestMsg::default().set_android(AndroidIcon));
}

/// - 长内容1
///
/// 会覆盖原有content
/// 只会推送多行信息
///
/// - 大图2
///
/// 会保留图片信息和原有content
/// 只能推送一张
///
/// - 横幅3
///
/// 可以传递多个，每个独立一行
/// 会隐藏原有content
///
#[derive(Debug)]
struct AndroidStyle;

impl Serialize for AndroidStyle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut style = serializer.serialize_struct("androidNotify", 2)?;
        // 1 长内容
        // {
        //     style.serialize_field(
        //         "content",
        //         &["555555555333333333333333333333333333333333333333333333\n欢迎来到小可食堂>>="],
        //     )?;
        //     style.serialize_field("style", &1)?;
        // }

        // 2 大图
        {
            style.serialize_field("content", &["https://i2.hdslb.com/bfs/archive/355b2e7886f337ff3d0951a057f0022be527f309.jpg@672w_378h_1c"])?;
            style.serialize_field("style", &2)?;
        }

        // // 3 横幅
        // {
        //     style.serialize_field("content", &["来点饼干<h1>嗯嗯</h1>","横幅干啥的？","不知道"])?;
        //     style.serialize_field("style", &3)?;
        // }
        style.end()
    }
}

#[test]
fn test_style() {
    test_pushing(|| TestMsg::default().set_android(AndroidStyle));
}

#[derive(Debug)]
struct Wrapper {
    custom_style: CustomStyle,
}

impl Serialize for Wrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut style = serializer.serialize_struct("androidNotify", 2)?;
        style.serialize_field("customStyle", &self.custom_style)?;
        style.serialize_field("style", &4)?;

        style.end()
    }
}

/// 用户定义的样式
/// 不知道啥用
/// 测试与普通推送几乎无区别
#[derive(Debug)]
struct CustomStyle;

impl Serialize for CustomStyle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut style = serializer.serialize_struct("CustomStyle", 2)?;

        style.serialize_field("styleNo", &2)?;

        style.serialize_field("buttonCopy", "open")?;

        style.serialize_field("buttonJumpUrl", "intent:https://www.bilibili.com/;end")?;

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

#[derive(Debug)]
struct IosBadge;

impl Serialize for IosBadge {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut badge = serializer.serialize_struct("IosNotify", 2)?;

        badge.serialize_field("badge", &12)?;
        badge.serialize_field("badgeType", &1)?;

        badge.end()
    }
}

#[test]
fn test_ios_badge() {
    test_pushing(|| TestMsg::default().set_ios(IosBadge));
}

#[derive(Debug)]
struct IosSubTitle;

impl Serialize for IosSubTitle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut sub_title = serializer.serialize_struct("IosNotify", 1)?;

        sub_title.serialize_field("subtitle", "小可试探副标题")?;

        sub_title.end()
    }
}

#[test]
fn test_ios_subtitle() {
    test_pushing(|| TestMsg::default().set_ios(IosSubTitle))
}

#[derive(Debug)]
struct IosSound;

impl Serialize for IosSound {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut sound = serializer.serialize_struct("IosNotify", 1)?;

        sound.serialize_field("sound", &())?;

        sound.end()
    }
}

#[test]
fn test_ios_no_sound() {
    test_pushing(|| TestMsg::default().set_ios(IosSound))
}

#[derive(Debug)]
struct RichText;

impl Serialize for RichText {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut rich = serializer.serialize_struct("IosNotify", 3)?;
        let img = "https://i2.hdslb.com/bfs/archive/a995572283104e306e433240b47fba772c4ed3a0.jpg@672w_378h_1c";
        rich.serialize_field("mutableContent", &1)?;
        rich.serialize_field("attachmentType", &1)?;
        rich.serialize_field("attachment", img)?;

        rich.end()
    }
}

#[test]
fn test_ios_rich() {
    test_pushing(|| TestMsg::default().set_ios(RichText))
}
