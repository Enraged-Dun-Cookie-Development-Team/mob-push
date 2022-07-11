mod push_model;
mod create_push;

use tokio::sync::mpsc;

use crate::{
    error::{self, MobPushError},
    UserSubscribeManage,
};

pub struct MobPusher<M: UserSubscribeManage> {
    manage: M,
    income_channel: mpsc::Receiver<M::PushData>,
    error_send: mpsc::Sender<error::MobPushError<M>>,
}

impl<M: UserSubscribeManage> MobPusher<M> {
    pub fn new(
        manage: M,
        buff_size: usize,
    ) -> (
        Self,
        mpsc::Sender<M::PushData>,
        mpsc::Receiver<MobPushError<M>>,
    ) {
        let (rx, tx) = mpsc::channel(buff_size);
        let (err_rx, err_tx) = mpsc::channel(16);
        (
            Self {
                manage,
                income_channel: tx,
                error_send: err_rx,
            },
            rx,
            err_tx,
        )
    }
}
