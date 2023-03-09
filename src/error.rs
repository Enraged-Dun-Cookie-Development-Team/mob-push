use std::fmt::{Debug, Display};

use crate::{http_client::PushClient, UserSubscribeManage};

/// mob push 推送期间的异常
pub enum MobPushError<M, C>
where
    M: UserSubscribeManage,
    C: PushClient,
{
    /// 用户订阅持久化管理出现的异常
    Manage(M::Err),
    /// 发起请求时异常
    Request(C::Error),
    /// json 序列化、反序列化异常
    Json(serde_json::Error),
    /// mob 推送响应异常
    Mob { state: u16, msg: String },
}

impl<M, C> std::fmt::Debug for MobPushError<M, C>
where
    M: UserSubscribeManage,
    C: PushClient,
    C::Error: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manage(err) => f.debug_tuple("Manage").field(err).finish(),
            Self::Request(err) => f.debug_tuple("Request").field(err).finish(),
            MobPushError::Mob { state, msg } => f
                .debug_struct("Mob")
                .field("state", state)
                .field("msg", msg)
                .finish(),
            MobPushError::Json(err) => f.debug_tuple("Json").field(err).finish(),
        }
    }
}

impl<M, C> std::fmt::Display for MobPushError<M, C>
where
    M: UserSubscribeManage,
    C: PushClient,
    C::Error: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MobPushError::Manage(err) => write!(f, "Subscribe Manage Error : {err}"),
            MobPushError::Request(err) => write!(f, "Request Error : {err}"),
            MobPushError::Mob { state, msg } => write!(f, "Mob Pusher Error : [{}] {}", state, msg),
            MobPushError::Json(err) => write!(f, "Json Error : {err}"),
        }
    }
}

impl<M, C> std::error::Error for MobPushError<M, C>
where
    M: UserSubscribeManage,
    C: PushClient,
    C::Error:std::error::Error
{
}

impl<M: UserSubscribeManage, C: PushClient> From<(u16, String)> for MobPushError<M, C> {
    fn from((state, msg): (u16, String)) -> Self {
        Self::Mob { state, msg }
    }
}

impl<M: UserSubscribeManage, C: PushClient> From<serde_json::Error> for MobPushError<M, C> {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}
