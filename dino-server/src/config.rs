use axum::http::Method;
use serde::Deserialize;

use crate::ProjectRoutes;

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub routes: ProjectRoutes,
}

#[derive(Debug, Deserialize)]
pub struct ProjectRoute {
    #[serde(deserialize_with = "deserialize_method")]
    pub method: Method,
    pub handler: String,
}

fn deserialize_method<'de, D>(deserializer: D) -> Result<Method, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    match s.to_uppercase().as_str() {
        "GET" => Ok(Method::GET),
        "POST" => Ok(Method::POST),
        "PUT" => Ok(Method::PUT),
        "DELETE" => Ok(Method::DELETE),
        "PATCH" => Ok(Method::PATCH),
        "HEAD" => Ok(Method::HEAD),
        "OPTIONS" => Ok(Method::OPTIONS),
        "TRACE" => Ok(Method::TRACE),
        "CONNECT" => Ok(Method::CONNECT),
        _ => Err(serde::de::Error::custom("invalid method")),
    }
}
