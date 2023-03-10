use std::{
    convert::Infallible,
    fmt::Debug,
    marker::{Send, Sync},
    time::Duration,
};

use mob_push::{
    self,
    http_client::{PushClient, PushRequestBuilder, PushResponse},
    load_config_from_default,
    push_notify::{
        android::{
            notify_style::CustomStyle, sound::WarnSound, AndroidNotify, Badge, Image, NotifyStyle,
        },
        ios::{IosBadgeType, IosNotify, IosPushSound, IosRichTextType},
    },
    MobPusher, PushEntity, SubscribeFilter, UserMobId, UserSubscribeManage,
};
use tokio::time;

struct Client(reqwest::Client);
struct RequestBuilder(reqwest::RequestBuilder);
struct Response(reqwest::Response);

impl PushClient for Client {
    type RequestBuilder = RequestBuilder;

    type Error = reqwest::Error;

    fn post(&self, url: impl Into<url::Url>) -> Self::RequestBuilder {
        RequestBuilder(self.0.post(url.into()))
    }

    fn send_request<'life0,'async_trait>(&'life0 self,req: <Self::RequestBuilder as mob_push::http_client::PushRequestBuilder> ::Request,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = Result< <Self::RequestBuilder as mob_push::http_client::PushRequestBuilder> ::Response,Self::Error> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait{
        Box::pin(async {
            let resp = self.0.execute(req).await?;
            Ok(Response(resp))
        })
    }
}

impl PushRequestBuilder for RequestBuilder {
    type Error = reqwest::Error;

    type Request = reqwest::Request;

    type Response = Response;

    fn header(self, key: &'static str, value: &str) -> Self {
        Self(self.0.header(key, value))
    }

    fn body(self, payload: Vec<u8>) -> Self {
        Self(self.0.body(payload))
    }

    fn build(self) -> Result<Self::Request, Self::Error> {
        self.0.build()
    }
}

impl PushResponse for Response {
    type Error = reqwest::Error;

    fn status(&self) -> u16 {
        self.0.status().as_u16()
    }

    fn bytes<'async_trait>(
        self,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = Result<Vec<u8>, Self::Error>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        Self: 'async_trait,
    {
        Box::pin(async { self.0.bytes().await.map(Into::into) })
    }
}

#[derive(Default)]
struct TestMsg {
    android: Option<Box<dyn Fn(&mut AndroidNotify) -> &mut AndroidNotify + Sync + Send + 'static>>,
    ios: Option<Box<dyn Fn(&mut IosNotify) -> &mut IosNotify + Sync + Send + 'static>>,
}

impl Debug for TestMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestMsg")
            .field("android", &self.android.is_some())
            .field("ios", &self.ios.is_some())
            .finish()
    }
}

impl TestMsg {
    fn set_android<AN>(self, android: AN) -> TestMsg
    where
        AN: Fn(&mut AndroidNotify) -> &mut AndroidNotify + Sync + Send + 'static,
    {
        let android = Box::new(android);
        TestMsg {
            android: Some(android),
            ios: self.ios,
        }
    }

    fn set_ios<IN>(self, ios: IN) -> TestMsg
    where
        IN: Fn(&mut IosNotify) -> &mut IosNotify + Sync + Send + 'static,
    {
        let ios = Box::new(ios);
        TestMsg {
            android: self.android,
            ios: Some(ios),
        }
    }
}

impl PushEntity for TestMsg {
    type Resource = i32;

    fn get_resource(&self) -> &Self::Resource {
        &11
    }

    type Content = str;

    fn get_send_content(&self) -> &Self::Content {
        "小刻食堂测试信息-----------------------------------------------------------------------很长"
    }

    fn get_title(&self) -> std::borrow::Cow<'_, str> {
        "新饼来袭".into()
    }

    fn android_notify(&self, notify: &mut AndroidNotify) {
        if let Some(an) = &self.android {
            an(notify);
        }
    }

    fn ios_notify(&self, notify: &mut mob_push::push_notify::ios::IosNotify) {
        if let Some(ios_notify) = &self.ios {
            ios_notify(notify);
        }
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

struct Manage;

#[async_trait::async_trait]
impl UserSubscribeManage for Manage {
    type UserIdentify = User;

    type PushData = TestMsg;

    type Filter = Filter;

    type Err = Infallible;

    async fn fetch_subscribe_filter(
        &self,
        _user_id: &Self::UserIdentify,
    ) -> Result<Self::Filter, Self::Err> {
        Ok(Filter)
    }

    async fn check_subscribed(
        &self,
        _user_id: &Self::UserIdentify,
        data_resource: &<Self::PushData as PushEntity>::Resource,
    ) -> Result<bool, Self::Err> {
        let resp = Filter.contains(data_resource);
        resp
    }

    async fn fetch_all_subscriber(
        &self,
        data_resource: &<Self::PushData as PushEntity>::Resource,
    ) -> Result<Vec<Self::UserIdentify>, Self::Err> {
        let resp = if data_resource == &11 || data_resource == &15 {
            vec![
                // User {
                //     mob_id: "65l063ct4qsghds".into(),
                // },
                // User {
                //     mob_id: "65kzw5w9iulerk0".into(),
                // },
                User {
                    mob_id: "65l05lvwtep0fls".into(),
                },
            ]
        } else {
            vec![]
        };

        Ok(resp)
    }
}

fn test_pushing<F>(msg: F)
where
    F: FnOnce() -> TestMsg,
{
    let client = reqwest::Client::new();
    load_config_from_default();
    let (mob_push, sender, mut err_rx) = MobPusher::new(Client(client), Manage, 8);

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
    test_pushing(|| TestMsg::default());
}

/// 角标数值没啥意义
/// 每次推送都是+1
#[test]
fn test_android_badge() {
    test_pushing(|| TestMsg::default().set_android(|an| an.set_badge(Badge::Add(1))));
}

/// 小米似乎只有一种声音？
#[test]
fn test_android_warn() {
    test_pushing(|| {
        TestMsg::default().set_android(|an| an.set_warn(WarnSound::Vibration & WarnSound::Prompt))
    });
}

/// 使用image 推送可行
#[test]
fn test_icon() {
    test_pushing(|| {
        TestMsg::default().set_android(|an| {
            let _img = "https://static.mob.com/www_mob_com/.nuxt/dist/client/img/62893e6.png";
            let icon = "https://www.mob.com/favicon.ico";

            an.set_image(Image::new_image(icon))
        })
    });
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
#[test]
fn test_style() {
    test_pushing(|| {
        TestMsg::default().set_android(|an| {
            let url = "https://i2.hdslb.com/bfs/archive/355b2e7886f337ff3d0951a057f0022be527f309.jpg@672w_378h_1c";
            an
                // .set_notify_style(NotifyStyle::new_long_content("555555555333333333333333333333333333333333333333333333\n欢迎来到小可食堂>>="))
//                 .set_notify_style(NotifyStyle::new_custom(CustomStyle::builder().button_copy("copy".to_string()).background_url("//i2.hdslb.com/bfs/archive/f84c09db27e1124ef3e88fb666a35335b140a857.jpg@672w_378h_1c_!web-home-common-cover".to_string())
// .button_jump_url("https://www.bilibili.com/".to_string())                
//                 .build()))
                // .set_notify_style(NotifyStyle::new_banner(["来点饼干<h1>嗯嗯</h1>","横幅干啥的？","不知道"]))
                .set_notify_style(NotifyStyle::new_big_vision(url))
        })
    });
}

#[test]
fn test_android_notify_push() {
    test_pushing(|| {
        TestMsg::default().set_android(|an|{
            an
            .set_notify_style(NotifyStyle::new_big_vision("https://i0.hdslb.com/bfs/archive/94bdaa89d9e1775f04bdfb705512a61e5de70628.jpg@672w_378h_1c"))
     .set_badge(Badge::new_add(1))
     .set_sound("114514".into())
     .set_warn(WarnSound::Prompt & WarnSound::IndicatorLight & WarnSound::Vibration)

    })
    })
}

#[test]
fn test_ios_badge() {
    test_pushing(|| TestMsg::default().set_ios(|ios| ios.set_badge(IosBadgeType::Abs(12))));
}

#[test]
fn test_ios_subtitle() {
    test_pushing(|| TestMsg::default().set_ios(|ios| ios.set_subtitle("小可试探副标题".into())))
}

#[test]
fn test_ios_no_sound() {
    test_pushing(|| TestMsg::default().set_ios(|ios| ios.set_sound(IosPushSound::None)))
}

#[test]
fn test_ios_rich() {
    let img = "https://i2.hdslb.com/bfs/archive/a995572283104e306e433240b47fba772c4ed3a0.jpg@672w_378h_1c";
    test_pushing(|| {
        TestMsg::default().set_ios(|ios| ios.set_rich_text(IosRichTextType::Picture(img.into())))
    })
}
