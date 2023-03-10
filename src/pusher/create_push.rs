use std::{ops::Deref, time::Duration};
use tokio::time::interval;
use tracing::{error, info, instrument};

use crate::{
    config::get_config,
    error::MobPushError,
    http_client::{PushClient, PushRequestBuilder, PushResponse},
    pusher::push_model::Forward,
    PushEntity, UserSubscribeManage,
};

use super::{
    push_model::{CreatePush, PushNotify, PushTarget, Respond},
    MobPusher,
};

impl<M: UserSubscribeManage, C: PushClient> MobPusher<M, C> {
    #[instrument(skip_all, name = "processPushing")]
    async fn pushing(
        client: &C,
        data: M::PushData,
        mut users: impl Iterator<Item = M::UserIdentify>,
    ) -> Result<(), MobPushError<M, C>> {
        let mut timer = interval(Duration::from_millis(500));
        while let Some(push_target) = PushTarget::new(&mut users) {
            let batch_size = push_target.target_user.len();
            // request body
            let body = CreatePush {
                push_target,
                push_notify: PushNotify::new_with_builder(&data),
                push_forward: Forward::new(&data),
            };

            let serde_body = serde_json::to_vec(&body)?;

            let md5_vec = {
                let mut temp = serde_body.clone();
                temp.extend(get_config().secret.as_bytes());
                temp
            };
            let md5_len = md5_vec.len();
            let md5 = md5::compute(md5_vec);

            info!(
                event = "Prepare to Push",
                users.batch_size = batch_size,
                push.payload.len = serde_body.len(),
                push.md5.len = md5_len,
                push.md5.value = format!("{md5:x}")
            );
            // request
            let req = client
                .post(url::Url::parse("http://api.push.mob.com/v3/push/createPush").unwrap())
                .default_headers()
                .header("sign", &format!("{md5:x}"))
                .body(serde_body)
                .build()
                .map_err(MobPushError::Request)?;

            let resp = client
                .send_request(req)
                .await
                .map_err(MobPushError::Request)?;

            // handle respond
            let resp = resp.bytes().await.map_err(MobPushError::Request)?;

            let resp: Respond = serde_json::from_slice(&resp)?;

            println!("{resp:?}");

            match resp.status {
                200 => {}
                state => {
                    let msg = resp.error.unwrap();
                    Err(MobPushError::Mob { state, msg })?;
                }
            };

            // delay
            timer.tick().await;
        }

        Ok(())
    }

    #[instrument(name = "PushTask", skip_all)]
    pub async fn start_up(mut self)
    where
        C::Error: std::error::Error,
    {
        let mut timer = interval(Duration::from_millis(500));
        while let Some(data) = self.income_channel.recv().await {
            info!(
                event = "PushData income",
                data.title = data.get_title().deref()
            );
            let error_sender = self.error_send.clone();
            let task = async {
                let subscribers = self.manage.fetch_all_subscriber(data.get_resource());
                let subscribers = subscribers.await.map_err(MobPushError::Manage)?;

                info!(
                    event = "finger out subscribers",
                    subscribers.len = subscribers.len()
                );
                Self::pushing(&self.client, data, subscribers.into_iter()).await?;
                Result::<_, MobPushError<_, _>>::Ok(())
            };
            match task.await {
                Ok(_) => {}
                Err(err) => {
                    error!(event="Error while Pushing",error = %err);
                    error_sender.send(err).await.expect("Receive half closed")
                }
            }
            timer.tick().await;
        }
    }
}
