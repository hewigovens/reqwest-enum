use crate::{http::HTTPResponse, target::Target};
use async_trait::async_trait;
use reqwest::{Client, Error};
use serde::de::DeserializeOwned;

#[async_trait]
pub trait ProviderType<T: Target> {
    async fn request(&self, target: T) -> Result<HTTPResponse, Error>;
}

#[async_trait]
pub trait JsonProviderType<T: Target>: ProviderType<T> {
    async fn request_json<U: DeserializeOwned>(&self, target: T) -> Result<U, Error>;
}

pub type EndpointFn<T> = fn(target: &T) -> String;
pub struct Provider<T: Target> {
    /// endpoint closure to customize the endpoint (url / path)
    endpoint_fn: Option<EndpointFn<T>>,
    client: Client,
}

#[async_trait]
impl<T> ProviderType<T> for Provider<T>
where
    T: Target + Send,
{
    async fn request(&self, target: T) -> Result<HTTPResponse, Error> {
        let url = self._request_url(&target);
        let mut request = self.client.request(target.method().into(), &url);
        let query_map = target.query();
        if !query_map.is_empty() {
            request = request.query(&query_map);
        }
        if !target.headers().is_empty() {
            for (k, v) in target.headers() {
                request = request.header(k, v);
            }
        }
        request.body(target.body().inner).send().await
    }
}

#[async_trait]
impl<T> JsonProviderType<T> for Provider<T>
where
    T: Target + Send,
{
    async fn request_json<U: DeserializeOwned>(&self, target: T) -> Result<U, Error> {
        let response = self.request(target).await?;
        let body = response.json::<U>().await?;
        Ok(body)
    }
}

impl<T> Provider<T>
where
    T: Target,
{
    pub fn new(endpoint_fn: EndpointFn<T>) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            endpoint_fn: Some(endpoint_fn),
        }
    }

    fn _request_url(&self, target: &T) -> String {
        let mut url = format!("{}{}", target.base_url(), target.path());
        if let Some(func) = &self.endpoint_fn {
            url = func(target);
        }
        url
    }
}

impl<T> Default for Provider<T>
where
    T: Target,
{
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint_fn: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        http::{HTTPBody, HTTPMethod},
        provider::Provider,
        target::Target,
    };
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize)]
    struct Person {
        name: String,
        age: u8,
        phones: Vec<String>,
    }

    enum HttpBin {
        Get,
        Post,
    }

    impl Target for HttpBin {
        fn base_url(&self) -> &'static str {
            "https://httpbin.org"
        }

        fn method(&self) -> HTTPMethod {
            match self {
                HttpBin::Get => HTTPMethod::GET,
                HttpBin::Post => HTTPMethod::POST,
            }
        }

        fn path(&self) -> &'static str {
            match self {
                HttpBin::Get => "/get",
                HttpBin::Post => "/post",
            }
        }

        fn query(&self) -> HashMap<&'static str, &'static str> {
            HashMap::default()
        }

        fn headers(&self) -> HashMap<&'static str, &'static str> {
            HashMap::default()
        }

        fn body(&self) -> HTTPBody {
            match self {
                HttpBin::Get => HTTPBody::default(),
                HttpBin::Post => HTTPBody::from(&Person {
                    name: "test".to_string(),
                    age: 20,
                    phones: vec!["1234567890".to_string()],
                }),
            }
        }
    }

    #[test]
    fn test_provider() {
        // test endpoint closure
        let provider = Provider::<HttpBin>::default();
        assert_eq!(
            provider._request_url(&HttpBin::Get),
            "https://httpbin.org/get"
        );

        let provider = Provider::<HttpBin>::new(|_: &HttpBin| "http://httpbin.org".to_string());
        assert_eq!(provider._request_url(&HttpBin::Post), "http://httpbin.org");
    }
}
