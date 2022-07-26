use crate::UserSubscribeManage;

/// mob push 推送期间的异常
pub enum MobPushError<M>
where
    M: UserSubscribeManage,
{
    /// 用户订阅持久化管理出现的异常
    Manage(M::Err),
    /// 发起请求时异常
    Reqwest(reqwest::Error),
    /// json 序列化、反序列化异常
    Json(serde_json::Error),
    /// mob 推送响应异常
    Mob { state: u16, msg: String },
}

impl<M> std::fmt::Debug for MobPushError<M>
where
    M: UserSubscribeManage,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manage(err) => f.debug_tuple("Manage").field(err).finish(),
            Self::Reqwest(err) => f.debug_tuple("Reqwest").field(err).finish(),
            MobPushError::Mob { state, msg } => f
                .debug_struct("Mob")
                .field("state", state)
                .field("msg", msg)
                .finish(),
            MobPushError::Json(err) => f.debug_tuple("Json").field(err).finish(),
        }
    }
}

impl<M: UserSubscribeManage> std::fmt::Display for MobPushError<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MobPushError::Manage(err) => write!(f, "Subscribe Manage Error : {err}"),
            MobPushError::Reqwest(err) => write!(f, "Reqwest Error : {err}"),
            MobPushError::Mob { state, msg } => write!(f, "Mob Pusher Error : [{}] {}", state, msg),
            MobPushError::Json(err) => write!(f, "Json Error : {err}"),
        }
    }
}

impl<M: UserSubscribeManage> std::error::Error for MobPushError<M> {}

impl<M: UserSubscribeManage> From<reqwest::Error> for MobPushError<M> {
    fn from(r: reqwest::Error) -> Self {
        MobPushError::Reqwest(r)
    }
}

impl<M: UserSubscribeManage> From<(u16, String)> for MobPushError<M> {
    fn from((state, msg): (u16, String)) -> Self {
        Self::Mob { state, msg }
    }
}

impl<M: UserSubscribeManage> From<serde_json::Error> for MobPushError<M> {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}
