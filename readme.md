# Mob Pusher

Mob 消息推送器

## Example

需要准备的类型

```rust
// 被推送的内容
struct PushingMessage{
    ...
}

impl PushEntity for PushingMessage {
    type Resource = ...;

    fn get_resource(&self) -> &Self::Resource {
        ...
    }

    type Content = ...;

    fn get_send_content(&self) -> &Self::Content {
        ...
    }

    fn get_title(&self) -> std::borrow::Cow<'_, str> {
        ...
    }

    fn android_notify(&self, notify: &mut AndroidNotify) {}

    fn ios_notify(&self, notify: &mut mob_push::push_notify::ios::IosNotify) {}
}

// 本地用户持久化信息获取
struct Manage{
    ...
};

#[async_trait]
impl UserSubscribeManage for Manage{
    type UserIdentify: User;
    type PushData: PushingMessage;
    type Filter: Filter;
    type Err: ...;

    async fn fetch_subscribe_filter(
        &self,
        user_id: &Self::UserIdentify,
    ) -> Result<Self::Filter, Self::Err>{
        ...
    }

    async fn check_subscribed(
        &self,
        user_id: &Self::UserIdentify,
        data_resource: &<Self::PushData as PushEntity>::Resource,
    ) -> Result<bool, Self::Err>{
        ...
    }

    async fn fetch_all_subscriber(
        &self,
        data_resource: &<Self::PushData as PushEntity>::Resource,
    ) -> Result<Vec<Self::UserIdentify>, Self::Err>{
        ...
    }
}

// 用户订阅信息筛选器
struct Filter{
    ...
};

impl SubscribeFilter for Filter{
    type Data = PushingMessage;

    type Err = ...;

    fn filter(
        &self,
        input: impl Iterator<Item = Self::Data>,
    ) -> Result<Vec<Self::Data>, Self::Err> {
       ...
    }

    fn contains(&self, target: &<Self::Data as PushEntity>::Resource) -> Result<bool, Self::Err> {
        ...
    }
}

// 订阅的用户
struct User{
    ...
}

impl UserMobId for User {
    type MobId = ...;

    fn get_mob_id(&self) -> Self::MobId {
        ...
    }
}

```

创建推送器

```rust
    let (
        // 推送器本体
        mob_push: MobPusher<Manage>, 
        // 推送消息发送端
        sender: tokio::mpsc::Sender<PushingMessage>, 
        // 推送器异常消息接收端
        mut err_rx: tokio::mpsc::Receiver<MobPushError<Manage>>
        ) = MobPusher::new(Manage::new(...), SIZE_OF_CHANNEL_BUFF);
```

启动推送器（需要在tokio异步运行时下）

```rust
    // 启动前先配置配置信息
    set_config(MobPushConfig{...});
    // 启动
    let join_handle: JoinHandle<()> = tokio::spawn(mob_push.start_up());

    // 推送消息
    sender.send(...).await.ok()

    // 接收异常消息
    while let Some(err) = err_rx.recv().await{
        // handle mob push error
    }
```
