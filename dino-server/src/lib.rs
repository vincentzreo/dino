mod config;
mod engine;
mod error;
mod middleware;
mod router;

use std::collections::HashMap;

use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Host, Query, State},
    http::{request::Parts, Response},
    response::IntoResponse,
    routing::any,
    Router,
};
use dashmap::DashMap;
use indexmap::IndexMap;
use matchit::Match;
use middleware::ServerTimeLayer;
use tokio::net::TcpListener;

pub use config::*;
pub use engine::*;
pub use error::AppError;
pub use router::*;
use tracing::info;

type ProjectRoutes = IndexMap<String, Vec<ProjectRoute>>;

#[derive(Clone)]
pub struct AppState {
    router: DashMap<String, SwappableAppRouter>,
}

#[derive(Clone)]
pub struct TenentRouter {
    host: String,
    router: SwappableAppRouter,
}

pub async fn start_server(port: u16, router: Vec<TenentRouter>) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    let map = DashMap::new();
    for r in router {
        map.insert(r.host, r.router);
    }
    let state = AppState::new(map);

    info!("Listening on {}", addr);
    let router = Router::new()
        .route("/*path", any(handler))
        .layer(ServerTimeLayer)
        .with_state(state);

    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}

#[allow(unused)]
async fn handler(
    State(state): State<AppState>,
    parts: Parts,
    Host(mut host): Host,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Bytes>,
) -> Result<impl IntoResponse, AppError> {
    // get router from state
    let router = get_router_by_host(host, state)?;
    // match router with parts.path get handler
    let matched = router.match_it(parts.method.clone(), parts.uri.path())?;
    let handler = matched.value;
    let req = assemble_req(&matched, &parts, query, body)?;
    // convert response into http response and return
    // TODO: build a worker pool, and send req via mpsc channel and get res from oneshot channel
    // but if code change, we need to restart the worker
    let worker = JsWorker::try_new(&router.code)?;
    let res = worker.run(handler, req)?;
    Ok(Response::from(res))
}

impl AppState {
    pub fn new(router: DashMap<String, SwappableAppRouter>) -> Self {
        Self { router }
    }
}

impl TenentRouter {
    pub fn new(host: impl Into<String>, router: SwappableAppRouter) -> Self {
        Self {
            host: host.into(),
            router,
        }
    }
}

fn get_router_by_host(mut host: String, state: AppState) -> Result<AppRouter, AppError> {
    host.truncate(host.find(':').unwrap_or(host.len()));

    Ok(state
        .router
        .get(&host)
        .ok_or(AppError::HostNotFound(host.to_string()))?
        .load())
}

fn assemble_req(
    matched: &Match<&str>,
    parts: &Parts,
    query: HashMap<String, String>,
    body: Option<Bytes>,
) -> Result<Req, AppError> {
    let params = matched
        .params
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<HashMap<_, _>>();
    // convert request data into Req and call handler with a js runtime
    let headers = parts
        .headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
        .collect::<HashMap<_, _>>();
    let body = body.and_then(|v| String::from_utf8(v.into()).ok());

    let req = Req::builder()
        .method(parts.method.to_string())
        .url(parts.uri.to_string())
        .headers(headers)
        .query(query)
        .params(params)
        .body(body)
        .build();
    Ok(req)
}
