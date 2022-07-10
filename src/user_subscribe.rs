use std::{future::Future, pin::Pin};

use crate::PushEntity;

pub trait UserSubscribeManage {
    type UserIdentify: Sized + 'static;
    type PushData: PushEntity;
    type Filter: SubscribeFilter;
    type Err: From<<Self::Filter as SubscribeFilter>::Err> + 'static;

    fn fetch_subscribe_filter(
        &self,
        user_id: &Self::UserIdentify,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Filter, Self::Err>> + Send + Sync>>;

    fn check_subscribed(
        &self,
        user_id: &Self::UserIdentify,
        data_group: &<Self::PushData as PushEntity>::Resource,
    ) -> Pin<Box<dyn Future<Output = Result<bool, Self::Err>> + Send + Sync>>;

    fn fetch_all_subscriber(
        &self,
        data_group: &<Self::PushData as PushEntity>::Resource,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Self::UserIdentify>, Self::Err>> + Send + Sync>>;
}

pub trait SubscribeFilter: 'static + Send + Sync {
    type Data: PushEntity;
    type Err: 'static;

    fn filter(input: impl Iterator<Item = Self::Data>) -> Result<Vec<Self::Data>, Self::Err>;

    fn contains(target: &<Self::Data as PushEntity>::Resource) -> Result<bool, Self::Err>;
}
