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
use crate::Error;
use std::time::Duration;
use std::future::Future;
use serde::de::DeserializeOwned;

#[cfg(feature = "middleware")]
use reqwest_middleware::{ClientBuilder as MiddlewareClientBuilder, ClientWithMiddleware};

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

#[derive(Debug)]
pub struct Provider<T: Target> {
    /// endpoint closure to customize the endpoint (url / path)
    endpoint_fn: Option<EndpointFn<T>>,
    request_fn: Option<RequestBuilderFn<T>>,
    timeout: Option<Duration>,
    #[cfg(not(feature = "middleware"))]
    client: reqwest::Client,
    #[cfg(feature = "middleware")]
    client: ClientWithMiddleware,
}

impl<T> ProviderType<T> for Provider<T>
where
    T: Target + Send,
{
    async fn request(&self, target: T) -> Result<HTTPResponse, Error> {
        let req = self.request_builder(&target)?.build()?;
        self.client.execute(req).await.map_err(Error::from)
    }
}

impl<T> JsonProviderType<T> for Provider<T>
where
    T: Target + Send,
{
    async fn request_json<U: DeserializeOwned>(&self, target: T) -> Result<U, Error> {
        let response = self.request(target).await?;

        // Check status and get Response or reqwest::Error
        let response = response.error_for_status()?;

        // If error_for_status succeeded, deserialize the JSON.
        let body: U = response.json().await?;

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

        let representative_target = &targets[0];

        let mut rb = self.request_builder(representative_target);

        let mut rpc_payload = Vec::new();
        for (k, individual_target) in targets.iter().enumerate() {
            let req = JsonRpcRequest::new(individual_target.method_name(), individual_target.params(), (k + 1) as u64);
            rpc_payload.push(req);
        }
        let body = HTTPBody::from_array(&rpc_payload).map_err(|e| JsonRpcError { code: -32700, message: format!("Failed to serialize batch request: {}", e) })?;

        rb = Ok(rb?.body(body.inner));

        // Build the final reqwest::Request
        let final_request = rb?.build().map_err(|e| JsonRpcError { code: -32603, message: format!("Failed to build batch request: {}", e) })?;

        // Execute the request using self.client
        let response = self.client.execute(final_request).await.map_err(|e| JsonRpcError { code: -32603, message: format!("Batch request execution failed: {}", e) })?;
        
        // Deserialize the response
        let response_body = response.json::<Vec<JsonRpcResult<U>>>().await.map_err(|e| JsonRpcError { code: -32700, message: format!("Failed to parse batch JSON response: {}", e) })?;
        Ok(response_body)
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

            let http_body = HTTPBody::from_array(&requests).map_err(|e| JsonRpcError { code: -32700, message: format!("Failed to serialize batch chunk: {}", e) })?;
            request = Ok(request?.body(http_body.inner));
            rpc_requests.push(request?);
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
        timeout: Option<Duration>,
    ) -> Self {
        #[cfg(not(feature = "middleware"))]
        let client = reqwest::Client::new();
        #[cfg(feature = "middleware")]
        let client = {
            MiddlewareClientBuilder::new(reqwest::Client::new()).build()
        };
        Self {
            client,
            endpoint_fn,
            request_fn,
            timeout,
        }
    }

    #[cfg(not(feature = "middleware"))]
    pub fn with_client(
        client: reqwest::Client,
        endpoint_fn: Option<EndpointFn<T>>,
        request_fn: Option<RequestBuilderFn<T>>,
    ) -> Self {
        Self {
            endpoint_fn,
            request_fn,
            client,
            timeout: None,
        }
    }

    #[cfg(feature = "middleware")]
    pub fn with_client(
        client: ClientWithMiddleware,
        endpoint_fn: Option<EndpointFn<T>>,
        request_fn: Option<RequestBuilderFn<T>>,
    ) -> Self {
        Self {
            endpoint_fn,
            request_fn,
            client,
            timeout: None,
        }
    }

    pub fn request_url(&self, target: &T) -> String {
        let mut url = format!("{}{}", target.base_url(), target.path());
        if let Some(func) = &self.endpoint_fn {
            url = func(target);
        }
        url
    }

    pub(crate) fn request_builder(&self, target: &T) -> Result<reqwest::RequestBuilder, Error> {
        let url = self.request_url(target);
        let temp_client = reqwest::Client::new();
        let mut request_builder = temp_client.request(target.method().into(), url.as_str());

        // apply query params
        request_builder = request_builder.query(&target.query());

        // apply headers
        for (key, value) in target.headers() {
            request_builder = request_builder.header(key, value);
        }

        // apply authentication
        if let Some(auth) = target.authentication() {
            request_builder = match auth {
                AuthMethod::Bearer(token) => request_builder.bearer_auth(token),
                AuthMethod::Basic(username, password) => request_builder.basic_auth(username, password),
                AuthMethod::Custom(auth_fn) => auth_fn(request_builder),
            };
        }

        // apply body
        let body = target.body()?;
        request_builder = request_builder.body(body.inner);

        // apply provider timeout
        if let Some(provider_timeout) = self.timeout {
            request_builder = request_builder.timeout(provider_timeout);
        }

        // apply request_fn closure
        if let Some(r_fn) = &self.request_fn {
            request_builder = r_fn(&request_builder, target);
        }

        Ok(request_builder)
    }
}

impl<T> Default for Provider<T>
where
    T: Target,
{
    fn default() -> Self {
        #[cfg(not(feature = "middleware"))]
        let client = reqwest::Client::new();
        #[cfg(feature = "middleware")]
        let client = {
            MiddlewareClientBuilder::new(reqwest::Client::new()).build()
        };
        Self {
            client,
            endpoint_fn: None,
            request_fn: None,
            timeout: None,
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
        HeaderAuth,
    }

    impl Target for HttpBin {
        fn base_url(&self) -> String {
            "https://httpbin.org".to_string()
        }

        fn method(&self) -> HTTPMethod {
            match self {
                HttpBin::Get => HTTPMethod::GET,
                HttpBin::Post => HTTPMethod::POST,
                HttpBin::Bearer => HTTPMethod::GET,
                HttpBin::HeaderAuth => HTTPMethod::GET,
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
                HttpBin::HeaderAuth => "/headers".into(),
            }
        }

        fn query(&self) -> HashMap<String, String> {
            HashMap::from([("foo".to_string(), "bar".to_string())])
        }

        fn headers(&self) -> HashMap<String, String> {
            HashMap::default()
        }

        fn authentication(&self) -> Option<AuthMethod> {
            match self {
                HttpBin::Bearer => Some(AuthMethod::Bearer("token".to_string())),
                HttpBin::HeaderAuth => Some(AuthMethod::header_api_key(
                    "X-Test-Api-Key".to_string(),
                    "my-secret-key".to_string(),
                )),
                _ => None,
            }
        }

        fn body(&self) -> Result<HTTPBody, crate::Error> {
            match self {
                HttpBin::Get | HttpBin::Bearer | HttpBin::HeaderAuth => Ok(HTTPBody::default()),
                HttpBin::Post => {
                    let person = Person {
                        name: "test".to_string(),
                        age: 20,
                        phones: vec!["1234567890".to_string()],
                    };
                    Ok(HTTPBody::from(&person)?)
                }
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
            Provider::<HttpBin>::new(Some(|_: &HttpBin| "http://httpbin.org".to_string()), None, None);
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
            None,
        );

        let request = provider.request_builder(&HttpBin::Get).unwrap().build().unwrap();
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

    #[test]
    fn test_header_api_key_auth() {
        let provider = Provider::<HttpBin>::default();
        block_on(async {
            let response: serde_json::Value = provider
                .request_json(HttpBin::HeaderAuth)
                .await
                .expect("request error");

            // httpbin /headers returns a JSON object like: {"headers": {"Header-Name": "Header-Value", ...}}
            let headers_map = response.get("headers").unwrap().as_object().unwrap();
            assert_eq!(
                headers_map.get("X-Test-Api-Key").unwrap().as_str().unwrap(),
                "my-secret-key"
            );
        });
    }
}
