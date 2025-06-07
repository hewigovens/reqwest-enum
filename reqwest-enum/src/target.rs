use crate::{
    Error,
    http::{AuthMethod, HTTPBody, HTTPMethod},
};
use std::collections::HashMap;

pub trait Target {
    fn base_url(&self) -> String;
    fn method(&self) -> HTTPMethod;
    fn path(&self) -> String;
    fn query(&self) -> HashMap<String, String>;
    fn headers(&self) -> HashMap<String, String>;
    /// Specifies the `AuthMethod` for the request, or `None` if no authentication.
    fn authentication(&self) -> Option<AuthMethod>;
    fn body(&self) -> Result<HTTPBody, Error>;

    // helpers for url
    fn query_string(&self) -> String {
        self.query()
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<String>>()
            .join("&")
    }

    fn absolute_url(&self) -> String {
        let mut url = format!("{}{}", self.base_url(), self.path());
        if !self.query_string().is_empty() {
            url = format!("{}?{}", url, self.query_string());
        }
        url
    }
}

#[cfg(feature = "jsonrpc")]
pub trait JsonRpcTarget: Target {
    fn method_name(&self) -> &'static str;
    fn params(&self) -> Vec<serde_json::Value>;
}
