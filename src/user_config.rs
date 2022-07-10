use std::{collections::HashSet, future::Future, pin::Pin};

use crate::PushEntity;

pub trait UserPushManage {
    type UserIdentify: 'static + Sized;
    type PushData: PushEntity + Sync + Send;
    type Err: 'static;

    fn fetch_user_filter(
        &self,
        user_id: &Self::UserIdentify,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<HashSet<<Self::PushData as PushEntity>::Group>, Self::Err>>
                + Send
                + Sync,
        >,
    >;

    fn check_push_need(
        &self,
        user_id: &Self::UserIdentify,
        data_group: <Self::PushData as PushEntity>::Group,
    ) -> Pin<Box<dyn Future<Output = Result<bool, Self::Err>> + Send + Sync>> {
        let set = self.fetch_user_filter(user_id);
        Box::pin(async move {
            let set = set.await?;
            Ok(set.contains(&data_group))
        })
    }

    fn fetch_all_need_push_user(
        &self,
        data_group: &<Self::PushData as PushEntity>::Group,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Self::UserIdentify>, Self::Err>> + Send + Sync>>;
}
