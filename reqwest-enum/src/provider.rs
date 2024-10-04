#[cfg(feature = "jsonrpc")]
use crate::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResult};
#[cfg(feature = "jsonrpc")]
use crate::target::JsonRpcTarget;
#[cfg(feature = "jsonrpc")]
use futures::future::join_all;

use crate::{
    http::{AuthMethod, HTTPBody, HTTPResponse},
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
pub type RequestBuilderFn<T> =
    fn(request_builder: &reqwest::RequestBuilder, target: &T) -> reqwest::RequestBuilder;
pub struct Provider<T: Target> {
    /// endpoint closure to customize the endpoint (url / path)
    endpoint_fn: Option<EndpointFn<T>>,
    request_fn: Option<RequestBuilderFn<T>>,
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
    pub fn new(
        endpoint_fn: Option<EndpointFn<T>>,
        request_fn: Option<RequestBuilderFn<T>>,
    ) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            endpoint_fn,
            request_fn,
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
        if let Some(auth) = target.authentication() {
            match auth {
                AuthMethod::Basic(username, password) => {
                    request = request.basic_auth(username, Some(password));
                }
                AuthMethod::Bearer(token) => {
                    request = request.bearer_auth(token);
                }
            }
        }
        if let Some(request_fn) = &self.request_fn {
            request = request_fn(&mut request, target);
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
            request_fn: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        http::{AuthMethod, HTTPBody, HTTPMethod},
        provider::{JsonProviderType, Provider},
        target::Target,
    };
    use serde::{Deserialize, Serialize};
    use std::collections::hash_map::DefaultHasher;
    use std::collections::HashMap;
    use std::hash::{Hash, Hasher};
    use std::time::{Duration, UNIX_EPOCH};
    use tokio_test::block_on;

    #[derive(Serialize, Deserialize)]
    struct Person {
        name: String,
        age: u8,
        phones: Vec<String>,
    }

    enum HttpBin {
        Get,
        Post,
        Bearer,
    }

    impl Target for HttpBin {
        fn base_url(&self) -> &'static str {
            "https://httpbin.org"
        }

        fn method(&self) -> HTTPMethod {
            match self {
                HttpBin::Get => HTTPMethod::GET,
                HttpBin::Post => HTTPMethod::POST,
                HttpBin::Bearer => HTTPMethod::GET,
            }
        }

        fn path(&self) -> String {
            let ts = UNIX_EPOCH + Duration::from_secs(1728044812);
            match self {
                HttpBin::Get => format!(
                    "/get?ts={}",
                    ts.duration_since(UNIX_EPOCH).unwrap().as_secs(),
                ),
                HttpBin::Post => "/post".into(),
                HttpBin::Bearer => "/bearer".into(),
            }
        }

        fn query(&self) -> HashMap<&'static str, &'static str> {
            HashMap::from([("foo", "bar")])
        }

        fn headers(&self) -> HashMap<&'static str, &'static str> {
            HashMap::default()
        }

        fn authentication(&self) -> Option<AuthMethod> {
            match self {
                HttpBin::Bearer => Some(AuthMethod::Bearer("token")),
                _ => None,
            }
        }

        fn body(&self) -> HTTPBody {
            match self {
                HttpBin::Get | HttpBin::Bearer => HTTPBody::default(),
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
            "https://httpbin.org/get?ts=1728044812"
        );

        let provider =
            Provider::<HttpBin>::new(Some(|_: &HttpBin| "http://httpbin.org".to_string()), None);
        assert_eq!(provider.request_url(&HttpBin::Post), "http://httpbin.org");
    }

    #[test]
    fn test_request_fn() {
        let provider = Provider::<HttpBin>::new(
            None,
            Some(|builder: &reqwest::RequestBuilder, target: &HttpBin| {
                let mut hasher = DefaultHasher::new();
                target.query_string().hash(&mut hasher);
                let hash = hasher.finish();

                let mut req = builder.try_clone().expect("trying to clone request");
                req = req.header("X-test", "test");
                req = req.header("X-hash", format!("{}", hash));
                req
            }),
        );

        let request = provider.request_builder(&HttpBin::Get).build().unwrap();
        let headers = request.headers();

        assert_eq!(request.method().to_string(), "GET");
        assert_eq!(headers.get("X-test").unwrap(), "test");
        assert_eq!(headers.get("X-hash").unwrap(), "3270317559611782182");
    }

    #[test]
    fn test_authentication() {
        let provider = Provider::<HttpBin>::default();
        block_on(async {
            let response: serde_json::Value = provider
                .request_json(HttpBin::Bearer)
                .await
                .expect("request error");

            assert!(response["authenticated"].as_bool().unwrap());
        });
    }
}
