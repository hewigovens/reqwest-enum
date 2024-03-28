use reqwest::{Body, Method};
use std::collections::HashMap;

pub trait Target {
    fn base_url(&self) -> &'static str;
    fn method(&self) -> Method;
    fn path(&self) -> &'static str;
    fn query(&self) -> HashMap<&'static str, &'static str>;
    fn headers(&self) -> HashMap<&'static str, &'static str>;
    fn body(&self) -> Body;
}
