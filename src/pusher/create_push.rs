use crate::{error::MobPushError, PushEntity, UserSubscribeManage};

use super::MobPusher;

impl<M: UserSubscribeManage> MobPusher<M> {
    async fn pushing(
        data: M::PushData,
        users: impl Iterator<Item = M::UserIdentify>,
    ) -> Result<(), reqwest::Error> {
        unimplemented!()
    }

    pub async fn start_up(mut self) {
        while let Some(data) = self.income_channel.recv().await {
            let subscribers = self.manage.fetch_all_subscriber(data.get_resource());
            let task = async move {
                let subscribe = subscribers.await.map_err(MobPushError::Manage)?;
                Self::pushing(data, subscribe.into_iter()).await?;
                Result::<_, MobPushError<M>>::Ok(())
            };
            let error_sender = self.error_send.clone();
            tokio::spawn(async move {
                match task.await {
                    Ok(_) => {}
                    Err(err) => error_sender.send(err).await.expect("Receive half closed"),
                }
            });
        }
    }
}
