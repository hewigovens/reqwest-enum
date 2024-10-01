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
    pub fn from<T>(value: &T) -> Self
    where
        T: serde::Serialize,
    {
        let mut bytes: Vec<u8> = Vec::new();
        serde_json::to_writer(&mut bytes, value).expect("serde_json serialize error");
        Self {
            inner: bytes.into(),
        }
    }

    pub fn from_array<T>(array: &[T]) -> Self
    where
        T: serde::Serialize,
    {
        let mut bytes: Vec<u8> = Vec::new();
        serde_json::to_writer(&mut bytes, array).expect("serde_json serialize error");
        Self {
            inner: bytes.into(),
        }
    }
}

pub enum AuthMethod {
    // Basic(username, password)
    Basic(&'static str, &'static str),
    // Bearer(token)
    Bearer(&'static str),
}
