use std::{future::Future, pin::Pin};

use crate::PushEntity;

type BoxResultFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send + Sync>>;

pub trait UserSubscribeManage: 'static {
    type UserIdentify: UserMobId;
    type PushData: PushEntity;
    type Filter: SubscribeFilter;
    type Err: From<<Self::Filter as SubscribeFilter>::Err>
        + std::error::Error
        + Send
        + Sync
        + 'static;

    fn fetch_subscribe_filter(
        &self,
        user_id: &Self::UserIdentify,
    ) -> BoxResultFuture<Self::Filter, Self::Err>;

    fn check_subscribed(
        &self,
        user_id: &Self::UserIdentify,
        data_resource: &<Self::PushData as PushEntity>::Resource,
    ) -> BoxResultFuture<bool, Self::Err>;

    fn fetch_all_subscriber(
        &self,
        data_resource: &<Self::PushData as PushEntity>::Resource,
    ) -> BoxResultFuture<Vec<Self::UserIdentify>, Self::Err>;
}

pub trait SubscribeFilter: 'static + Send + Sync {
    type Data: PushEntity;
    type Err: 'static;

    fn filter(input: impl Iterator<Item = Self::Data>) -> Result<Vec<Self::Data>, Self::Err>;

    fn contains(target: &<Self::Data as PushEntity>::Resource) -> Result<bool, Self::Err>;
}

pub trait UserMobId: Sized + Send + Sync + 'static {
    type MobId: ToString + 'static + Send + Sync + Sized;

    fn get_mob_id(&self) -> Self::MobId;
}