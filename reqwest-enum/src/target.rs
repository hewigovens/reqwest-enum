use crate::http::{AuthMethod, HTTPBody, HTTPMethod};
use std::collections::HashMap;
pub trait Target {
    fn base_url(&self) -> &'static str;
    fn method(&self) -> HTTPMethod;
    fn path(&self) -> &'static str;
    fn query(&self) -> HashMap<&'static str, &'static str>;
    fn headers(&self) -> HashMap<&'static str, &'static str>;
    fn authentication(&self) -> Option<AuthMethod>;
    fn body(&self) -> HTTPBody;

    // helpers for url
    fn query_string(&self) -> String {
        self.query()
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
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
