use std::sync::Arc;

use anyhow::Result;
use arc_swap::ArcSwap;
use axum::http::Method;
use matchit::{Match, Router};

use crate::{AppError, ProjectRoutes};

#[derive(Clone)]
pub struct SwappableAppRouter {
    pub routers: Arc<ArcSwap<Router<MethodRoute>>>,
}

#[derive(Clone)]
pub struct AppRouter(Arc<Router<MethodRoute>>);

#[derive(Debug, Clone, Default)]
pub struct MethodRoute {
    get: Option<String>,
    head: Option<String>,
    delete: Option<String>,
    options: Option<String>,
    patch: Option<String>,
    post: Option<String>,
    put: Option<String>,
    trace: Option<String>,
    connect: Option<String>,
}

impl SwappableAppRouter {
    pub fn try_new(routers: ProjectRoutes) -> Result<Self> {
        let router = Self::get_router(routers)?;
        Ok(Self {
            routers: Arc::new(ArcSwap::from_pointee(router)),
        })
    }
    pub fn swap(&self, routers: ProjectRoutes) -> Result<()> {
        let router = Self::get_router(routers)?;
        self.routers.store(Arc::new(router));
        Ok(())
    }
    pub fn load(&self) -> AppRouter {
        AppRouter(self.routers.load_full())
    }
    fn get_router(routers: ProjectRoutes) -> Result<Router<MethodRoute>> {
        let mut router = Router::new();
        for (path, routes) in routers {
            let mut method_route = MethodRoute::default();
            for route in routes {
                match route.method {
                    Method::GET => method_route.get = Some(route.handler),
                    Method::HEAD => method_route.head = Some(route.handler),
                    Method::DELETE => method_route.delete = Some(route.handler),
                    Method::OPTIONS => method_route.options = Some(route.handler),
                    Method::PATCH => method_route.patch = Some(route.handler),
                    Method::POST => method_route.post = Some(route.handler),
                    Method::PUT => method_route.put = Some(route.handler),
                    Method::TRACE => method_route.trace = Some(route.handler),
                    Method::CONNECT => method_route.connect = Some(route.handler),
                    v => unreachable!("Unsupported method: {:?}", v),
                }
            }
            router.insert(path, method_route)?;
        }
        Ok(router)
    }
}

impl AppRouter {
    pub fn match_it<'m, 'p>(
        &'m self,
        method: Method,
        path: &'p str,
    ) -> Result<Match<&str>, AppError>
    where
        'p: 'm,
    {
        let Ok(ret) = self.0.at(path) else {
            return Err(AppError::RoutePathNotFound(path.to_string()));
        };
        let s = match method {
            Method::GET => ret.value.get.as_deref(),
            Method::HEAD => ret.value.head.as_deref(),
            Method::DELETE => ret.value.delete.as_deref(),
            Method::OPTIONS => ret.value.options.as_deref(),
            Method::PATCH => ret.value.patch.as_deref(),
            Method::POST => ret.value.post.as_deref(),
            Method::PUT => ret.value.put.as_deref(),
            Method::TRACE => ret.value.trace.as_deref(),
            Method::CONNECT => ret.value.connect.as_deref(),
            _ => unreachable!(),
        }
        .ok_or_else(|| AppError::RouteMethodNotAllowed(method))?;
        Ok(Match {
            value: s,
            params: ret.params,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ProjectConfig;

    use super::*;

    #[test]
    fn app_router_match_should_work() {
        let config = include_str!("../fixtures/config.yml");
        let project_config: ProjectConfig = serde_yaml::from_str(config).unwrap();
        let router = SwappableAppRouter::try_new(project_config.routes).unwrap();
        let app_router = router.load();
        let m = app_router.match_it(Method::GET, "/api/hello/1").unwrap();
        assert_eq!(m.value, "hello");
        assert_eq!(m.params.get("id"), Some("1"));

        let m = app_router.match_it(Method::POST, "/api/zzq/2").unwrap();
        assert_eq!(m.value, "hello4");
        assert_eq!(m.params.get("id"), Some("2"));
        assert_eq!(m.params.get("name"), Some("zzq"));
    }

    #[test]
    fn app_router_swap_should_work() {
        let config = include_str!("../fixtures/config.yml");
        let project_config: ProjectConfig = serde_yaml::from_str(config).unwrap();
        let router = SwappableAppRouter::try_new(project_config.routes).unwrap();
        let app_router = router.load();
        let m = app_router.match_it(Method::GET, "/api/hello/1").unwrap();
        assert_eq!(m.value, "hello");
        assert_eq!(m.params.get("id"), Some("1"));

        let new_config = include_str!("../fixtures/config1.yml");
        let new_project_config: ProjectConfig = serde_yaml::from_str(new_config).unwrap();
        router.swap(new_project_config.routes).unwrap();
        let app_router = router.load();
        let m = app_router.match_it(Method::POST, "/api/zzq/2").unwrap();
        assert_eq!(m.value, "handler2");
        assert_eq!(m.params.get("id"), Some("2"));
        assert_eq!(m.params.get("name"), Some("zzq"));
    }
}
