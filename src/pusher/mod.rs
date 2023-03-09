mod create_push;
mod push_model;

use tokio::sync::mpsc;

use crate::{
    error::{self, MobPushError},
    http_client::PushClient,
    UserSubscribeManage,
};

/// mob push 推送器
pub struct MobPusher<M: UserSubscribeManage, C: PushClient> {
    manage: M,
    client: C,
    income_channel: mpsc::Receiver<M::PushData>,
    error_send: mpsc::Sender<error::MobPushError<M, C>>,
}

impl<M: UserSubscribeManage, C: PushClient> MobPusher<M, C> {
    pub fn new(
        client: C,
        manage: M,
        buff_size: usize,
    ) -> (
        Self,
        mpsc::Sender<M::PushData>,
        mpsc::Receiver<MobPushError<M, C>>,
    ) {
        let (rx, tx) = mpsc::channel(buff_size);
        let (err_rx, err_tx) = mpsc::channel(16);
        (
            Self {
                manage,
                income_channel: tx,
                error_send: err_rx,
                client,
            },
            rx,
            err_tx,
        )
    }
}
