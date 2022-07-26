use crate::PushEntity;
use async_trait::async_trait;

/// 用户消息订阅管理器, 负责管理mob push 用户订阅的持久化数据获取
#[async_trait]
pub trait UserSubscribeManage: 'static + Sync + Send {
    type UserIdentify: UserMobId;
    type PushData: PushEntity;
    type Filter: SubscribeFilter;
    type Err: From<<Self::Filter as SubscribeFilter>::Err>
        + std::error::Error
        + Send
        + Sync
        + 'static;

    /// 获取指定用户的订阅来源筛选器
    async fn fetch_subscribe_filter(
        &self,
        user_id: &Self::UserIdentify,
    ) -> Result<Self::Filter, Self::Err>;

    /// 检查某一用户是否订阅了指定数据源
    async fn check_subscribed(
        &self,
        user_id: &Self::UserIdentify,
        data_resource: &<Self::PushData as PushEntity>::Resource,
    ) -> Result<bool, Self::Err>;

    /// 获取全部订阅了指定数据源的用户
    async fn fetch_all_subscriber(
        &self,
        data_resource: &<Self::PushData as PushEntity>::Resource,
    ) -> Result<Vec<Self::UserIdentify>, Self::Err>;
}

/// 订阅用户筛选器
pub trait SubscribeFilter: 'static + Send + Sync {
    /// 要推送的信息
    type Data: PushEntity;
    /// 筛选器异常
    type Err: 'static;

    fn filter(&self, input: impl Iterator<Item = Self::Data>)
        -> Result<Vec<Self::Data>, Self::Err>;

    fn contains(&self, target: &<Self::Data as PushEntity>::Resource) -> Result<bool, Self::Err>;
}

/// 用户信息
pub trait UserMobId: Sized + Send + Sync + 'static {
    type MobId: ToString + 'static + Send + Sync + Sized;
    /// 用户推送用mob ID
    fn get_mob_id(&self) -> Self::MobId;
}
