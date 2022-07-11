use std::time::Duration;

use once_cell::sync::OnceCell;
use reqwest::{header::HeaderMap, Client};
use tokio::time::interval;

use crate::{config::get_config, error::MobPushError, PushEntity, UserSubscribeManage};

use super::{
    push_model::{CreatePush, PushNotify, PushTarget, Respond},
    MobPusher,
};

static CLIENT: OnceCell<Client> = OnceCell::new();

fn get_client() -> Result<&'static Client, reqwest::Error> {
    CLIENT.get_or_try_init(|| {
        let mut headers = HeaderMap::new();
        headers.append(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        headers.append("key", get_config().key.parse().unwrap());

        Client::builder().default_headers(headers).build()
    })
}

impl<M: UserSubscribeManage> MobPusher<M> {
    async fn pushing(
        data: M::PushData,
        mut users: impl Iterator<Item = M::UserIdentify>,
    ) -> Result<(), MobPushError<M>> {
        let client = get_client()?;

        let mut timer = interval(Duration::from_millis(500));
        while let Some(push_target) = PushTarget::new(&mut users) {
            // request body
            let body = CreatePush {
                push_target,
                push_notify: PushNotify::new(&data)?,
            };

            let serde_body = serde_json::to_vec(&body)?;
            let md5_vec = {
                let mut temp = serde_body.clone();
                temp.extend(get_config().secret.as_bytes());
                temp
            };
            let md5 = md5::compute(md5_vec);

            // request
            let resp = client
                .post("http://api.push.mob.com/v3/push/createPush")
                .header("sign", &format!("{md5:x}"))
                .send()
                .await?;

            // handle respond
            let resp = resp.bytes().await?;

            let resp: Respond = serde_json::from_slice(&resp)?;

            match resp.status {
                200 => {}
                state => {
                    let msg = resp.error.unwrap();
                    Err(MobPushError::<M>::Mob { state, msg })?;
                }
            };

            // delay
            timer.tick().await;
        }

        Ok(())
    }

    pub async fn start_up(mut self) {
        while let Some(data) = self.income_channel.recv().await {
            let subscribers = self.manage.fetch_all_subscriber(data.get_resource());
            let task = async move {
                let subscribers = subscribers.await.map_err(MobPushError::Manage)?;
                Self::pushing(data, subscribers.into_iter()).await?;
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
