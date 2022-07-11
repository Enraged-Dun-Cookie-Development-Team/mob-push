use crate::UserSubscribeManage;

pub enum MobPushError<M>
where
    M: UserSubscribeManage,
{
    Manage(M::Err),
    Reqwest(reqwest::Error),
}

impl<M> std::fmt::Debug for MobPushError<M>
where
    M: UserSubscribeManage,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manage(arg0) => f.debug_tuple("Manage").field(arg0).finish(),
            Self::Reqwest(arg0) => f.debug_tuple("Reqwest").field(arg0).finish(),
        }
    }
}

impl<M: UserSubscribeManage> std::fmt::Display for MobPushError<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MobPushError::Manage(err) => write!(f, "Subscribe Manage Error : {err}"),
            MobPushError::Reqwest(err) => write!(f, "Reqwest Error : {err}"),
        }
    }
}

impl<M: UserSubscribeManage> std::error::Error for MobPushError<M> {}

impl<M: UserSubscribeManage> From<reqwest::Error> for MobPushError<M> {
    fn from(r: reqwest::Error) -> Self {
        MobPushError::Reqwest(r)
    }
}

