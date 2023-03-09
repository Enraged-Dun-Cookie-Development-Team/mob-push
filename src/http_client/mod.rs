use async_trait::async_trait;

use crate::config::get_config;

#[async_trait]
pub trait PushClient: Sized {
    type RequestBuilder: PushRequestBuilder<Error = Self::Error>;
    type Error;

    fn post(&self, url: impl Into<url::Url>) -> Self::RequestBuilder;

    async fn send_request(
        &self,
        req: <Self::RequestBuilder as PushRequestBuilder>::Request,
    ) -> Result<<Self::RequestBuilder as PushRequestBuilder>::Response, Self::Error>;
}

pub trait PushRequestBuilder: Sized {
    type Error;
    type Request;
    type Response: PushResponse<Error = Self::Error>;

    fn default_headers(self) -> Self {
        self.header("content-type", "application/json")
            .header("key", &get_config().key)
    }

    fn header(self, key: &'static str, value: &str) -> Self;
    fn body(self, payload: Vec<u8>) -> Self;
    fn build(self) -> Result<Self::Request, Self::Error>;
}

#[async_trait]
pub trait PushResponse {
    type Error;
    fn status(&self) -> u16;
    async fn bytes(self) -> Result<Vec<u8>, Self::Error>;
}
