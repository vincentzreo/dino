use std::collections::HashMap;

use anyhow::Result;
use axum::{body::Body, response::Response};
use dino_macros::{FromJs, IntoJs};
use rquickjs::{Context, Function, Object, Promise, Runtime};
use typed_builder::TypedBuilder;

#[allow(unused)]
pub struct JsWorker {
    rt: Runtime,
    ctx: Context,
}

fn print(msg: String) {
    println!("{}", msg);
}

#[derive(Debug, TypedBuilder, IntoJs)]
pub struct Req {
    #[builder(setter(into))]
    pub method: String,
    #[builder(setter(into))]
    pub url: String,
    #[builder(default)]
    pub query: HashMap<String, String>,
    #[builder(default)]
    pub params: HashMap<String, String>,
    #[builder(default)]
    pub headers: HashMap<String, String>,
    #[builder(default)]
    pub body: Option<String>,
}

#[derive(Debug, FromJs)]
pub struct Res {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl JsWorker {
    pub fn try_new(module: &str) -> Result<Self> {
        let rt = Runtime::new()?;
        let ctx = Context::full(&rt)?;
        ctx.with(|ctx| {
            let global = ctx.globals();
            let ret: Object = ctx.eval(module)?;
            global.set("handlers", ret)?;
            // set up the print function
            global.set(
                "print",
                Function::new(ctx.clone(), print)?.with_name("print"),
            )?;
            Ok::<_, anyhow::Error>(())
        })?;
        Ok(Self { rt, ctx })
    }

    pub fn run(&self, name: &str, req: Req) -> Result<Res> {
        self.ctx.with(|ctx| {
            let global = ctx.globals();
            let handlers: Object = global.get("handlers")?;
            let func: Function = handlers.get(name)?;
            let v: Promise = func.call((req,))?;
            Ok::<_, anyhow::Error>(v.finish()?)
        })
    }
}

impl From<Res> for Response {
    fn from(value: Res) -> Self {
        let mut builder = Response::builder().status(value.status);
        for (k, v) in value.headers {
            builder = builder.header(k, v);
        }
        if let Some(body) = value.body {
            builder.body(body.into()).unwrap()
        } else {
            builder.body(Body::empty()).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_worker_should_run() {
        let code = r#"
        (function(){async function hello(req){return{status:200,headers:{"content-type":"application/json"},body: JSON.stringify(req)};}return{hello:hello};})();"#;

        let req = Req::builder()
            .method("GET")
            .url("http://localhost:8080")
            .headers(HashMap::new())
            .build();
        let worker = JsWorker::try_new(code).unwrap();
        let ret = worker.run("hello", req).unwrap();
        assert_eq!(ret.status, 200);
    }
}
