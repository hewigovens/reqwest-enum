use crate::target::Target;
use async_trait::async_trait;
use reqwest::{Client, Error, Response};
use std::marker::PhantomData;

#[async_trait(?Send)]
pub trait ProviderType<T: Target> {
    async fn request(&self, target: T) -> Result<Response, Error>;
}

pub struct Provider<T: Target> {
    client: Client,
    __phantom: PhantomData<T>,
}

#[async_trait(?Send)]
impl<T> ProviderType<T> for Provider<T>
where
    T: Target,
{
    async fn request(&self, target: T) -> Result<Response, Error> {
        let url = format!("{}{}", target.base_url(), target.path());
        let mut request = self.client.request(target.method(), &url);
        let query_map = target.query();
        if !query_map.is_empty() {
            request = request.query(&query_map);
        }
        if !target.headers().is_empty() {
            for (k, v) in target.headers() {
                request = request.header(k, v);
            }
        }
        request.body(target.body()).send().await
    }
}

impl<T> Default for Provider<T>
where
    T: Target,
{
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            __phantom: PhantomData,
        }
    }
}
