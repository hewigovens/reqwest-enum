use reqwest::Method;

#[derive(Debug, Default)]
pub struct HTTPBody {
    pub inner: reqwest::Body,
}

impl HTTPBody {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().unwrap_or_default().to_vec()
    }
}

pub type HTTPResponse = reqwest::Response;

pub enum HTTPMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
    CONNECT,
}

impl std::fmt::Display for HTTPMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            HTTPMethod::GET => "GET",
            HTTPMethod::POST => "POST",
            HTTPMethod::PUT => "PUT",
            HTTPMethod::DELETE => "DELETE",
            HTTPMethod::PATCH => "PATCH",
            HTTPMethod::OPTIONS => "OPTIONS",
            HTTPMethod::HEAD => "HEAD",
            HTTPMethod::CONNECT => "CONNECT",
        };
        write!(f, "{}", s)
    }
}

impl From<HTTPMethod> for Method {
    fn from(val: HTTPMethod) -> Self {
        match val {
            HTTPMethod::GET => Method::GET,
            HTTPMethod::POST => Method::POST,
            HTTPMethod::PUT => Method::PUT,
            HTTPMethod::DELETE => Method::DELETE,
            HTTPMethod::PATCH => Method::PATCH,
            HTTPMethod::OPTIONS => Method::OPTIONS,
            HTTPMethod::HEAD => Method::HEAD,
            HTTPMethod::CONNECT => Method::CONNECT,
        }
    }
}

impl HTTPBody {
    pub fn from<S: serde::Serialize>(value: &S) -> Result<Self, serde_json::Error> {
        let mut writer = Vec::new();
        serde_json::to_writer(&mut writer, value)?;
        Ok(Self { inner: writer.into() })
    }

    pub fn from_array<S: serde::Serialize>(array: &[S]) -> Result<Self, serde_json::Error> {
        let mut writer = Vec::new();
        serde_json::to_writer(&mut writer, array)?;
        Ok(Self { inner: writer.into() })
    }
}

/// Authentication method for a request (Basic, Bearer, or Custom closure).
pub enum AuthMethod {
    /// HTTP Basic authentication.
    /// Takes a username (`String`) and an optional password (`Option<String>`).
    Basic(String, Option<String>),
    /// HTTP Bearer authentication.
    /// Takes a token (`String`).
    Bearer(String),
    /// Custom authentication logic provided as a closure.
    /// The closure takes a `reqwest::RequestBuilder` and returns a modified `reqwest::RequestBuilder`.
    /// This allows for flexible and complex authentication mechanisms.
    /// 
    /// # Example
    ///
    /// ```rust,ignore
    /// AuthMethod::Custom(Box::new(|rb| rb.header("X-Custom-Auth", "some_value")))
    /// ```
    Custom(Box<dyn Fn(reqwest::RequestBuilder) -> reqwest::RequestBuilder + Send + Sync + 'static>),
}

impl AuthMethod {
    /// Helper for `AuthMethod::Custom`: API key authentication via a request header.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use reqwest_enum::http::AuthMethod;
    ///
    /// let auth = AuthMethod::header_api_key("Authorization".to_string(), "mysecretapikey".to_string());
    /// // This can then be returned by a Target's `authentication()` method.
    /// ```
    pub fn header_api_key(header_name: String, api_key: String) -> Self {
        AuthMethod::Custom(Box::new(
            move |rb: reqwest::RequestBuilder| {
                rb.header(header_name.clone(), api_key.clone())
            },
        ))
    }
}

impl std::fmt::Debug for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMethod::Basic(username, password) => f
                .debug_tuple("Basic")
                .field(username)
                .field(password)
                .finish(),
            AuthMethod::Bearer(token) => f.debug_tuple("Bearer").field(token).finish(),
            AuthMethod::Custom(_) => f.debug_tuple("Custom").field(&"<function>").finish(),
        }
    }
}
