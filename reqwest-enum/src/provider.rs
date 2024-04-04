#[cfg(feature = "jsonrpc")]
use crate::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResult, JsonRpcTarget};
#[cfg(feature = "jsonrpc")]
use futures::future::join_all;

use crate::{
    http::{HTTPBody, HTTPResponse},
    target::Target,
};
use core::future::Future;
use reqwest::{Client, Error};
use serde::de::DeserializeOwned;

pub trait ProviderType<T: Target>: Send {
    /// request to target and return http response
    fn request(&self, target: T) -> impl Future<Output = Result<HTTPResponse, Error>>;
}

pub trait JsonProviderType<T: Target>: ProviderType<T> {
    /// request and deserialize response to json using serde
    fn request_json<U: DeserializeOwned>(
        &self,
        target: T,
    ) -> impl Future<Output = Result<U, Error>>;
}

#[cfg(feature = "jsonrpc")]

pub trait JsonRpcProviderType<T: Target>: ProviderType<T> {
    /// batch isomorphic JSON-RPC requests
    fn batch<U: DeserializeOwned>(
        &self,
        targets: Vec<T>,
    ) -> impl Future<Output = Result<Vec<JsonRpcResult<U>>, JsonRpcError>>;

    fn batch_chunk_by<U: DeserializeOwned>(
        &self,
        targets: Vec<T>,
        chunk_size: usize,
    ) -> impl Future<Output = Result<Vec<JsonRpcResult<U>>, JsonRpcError>>;
}

pub type EndpointFn<T> = fn(target: &T) -> String;
pub struct Provider<T: Target> {
    /// endpoint closure to customize the endpoint (url / path)
    endpoint_fn: Option<EndpointFn<T>>,
    client: Client,
}

impl<T> ProviderType<T> for Provider<T>
where
    T: Target + Send,
{
    async fn request(&self, target: T) -> Result<HTTPResponse, Error> {
        let mut request = self.request_builder(&target);
        request = request.body(target.body().inner);
        request.send().await
    }
}

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

#[cfg(feature = "jsonrpc")]
impl<T> JsonRpcProviderType<T> for Provider<T>
where
    T: JsonRpcTarget + Send,
{
    async fn batch<U: DeserializeOwned>(
        &self,
        targets: Vec<T>,
    ) -> Result<Vec<JsonRpcResult<U>>, JsonRpcError> {
        if targets.is_empty() {
            return Err(JsonRpcError {
                code: -32600,
                message: "Invalid Request".into(),
            });
        }

        let target = &targets[0];
        let mut request = self.request_builder(target);
        let mut requests = Vec::<JsonRpcRequest>::new();
        for (k, v) in targets.iter().enumerate() {
            let request = JsonRpcRequest::new(v.method_name(), v.params(), (k + 1) as u64);
            requests.push(request);
        }

        request = request.body(HTTPBody::from_array(&requests).inner);
        let response = request.send().await?;
        let body = response.json::<Vec<JsonRpcResult<U>>>().await?;
        Ok(body)
    }

    async fn batch_chunk_by<U: DeserializeOwned>(
        &self,
        targets: Vec<T>,
        chunk_size: usize,
    ) -> Result<Vec<JsonRpcResult<U>>, JsonRpcError> {
        if targets.is_empty() || chunk_size == 0 {
            return Err(JsonRpcError {
                code: -32600,
                message: "Invalid Request".into(),
            });
        }

        let chunk_targets = targets.chunks(chunk_size).collect::<Vec<_>>();
        let mut rpc_requests = Vec::<reqwest::RequestBuilder>::new();

        for (chunk_idx, chunk) in chunk_targets.into_iter().enumerate() {
            let target = &chunk[0];
            let mut request = self.request_builder(target);
            let mut requests = Vec::<JsonRpcRequest>::new();
            for (k, v) in chunk.iter().enumerate() {
                let request = JsonRpcRequest::new(
                    v.method_name(),
                    v.params(),
                    (chunk_idx * chunk_size + k + 1) as u64,
                );
                requests.push(request);
            }

            request = request.body(HTTPBody::from_array(&requests).inner);
            rpc_requests.push(request);
        }
        let bodies = join_all(rpc_requests.into_iter().map(|request| async move {
            let response = request.send().await?;
            let body = response.json::<Vec<JsonRpcResult<U>>>().await?;
            Ok(body)
        }))
        .await;

        let mut results = Vec::<JsonRpcResult<U>>::new();
        let mut error: Option<JsonRpcError> = None;

        for result in bodies {
            match result {
                Ok(body) => {
                    results.extend(body);
                }
                Err(err) => {
                    error = Some(err);
                }
            }
        }
        if let Some(err) = error {
            return Err(err);
        }
        Ok(results)
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

    pub(crate) fn request_url(&self, target: &T) -> String {
        let mut url = format!("{}{}", target.base_url(), target.path());
        if let Some(func) = &self.endpoint_fn {
            url = func(target);
        }
        url
    }

    pub(crate) fn request_builder(&self, target: &T) -> reqwest::RequestBuilder {
        let url = self.request_url(target);
        let mut request = self.client.request(target.method().into(), url);
        let query_map = target.query();
        if !query_map.is_empty() {
            request = request.query(&query_map);
        }
        if !target.headers().is_empty() {
            for (k, v) in target.headers() {
                request = request.header(k, v);
            }
        }
        request
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
    fn test_test_endpoint_closure() {
        let provider = Provider::<HttpBin>::default();
        assert_eq!(
            provider.request_url(&HttpBin::Get),
            "https://httpbin.org/get"
        );

        let provider = Provider::<HttpBin>::new(|_: &HttpBin| "http://httpbin.org".to_string());
        assert_eq!(provider.request_url(&HttpBin::Post), "http://httpbin.org");
    }
}
