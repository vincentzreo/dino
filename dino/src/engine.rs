use std::collections::HashMap;

use anyhow::Result;
use rquickjs::{Context, FromJs, Function, IntoJs, Object, Promise, Runtime};
use typed_builder::TypedBuilder;

#[allow(unused)]
pub struct JsWorker {
    rt: Runtime,
    ctx: Context,
}

fn print(msg: String) {
    println!("{}", msg);
}

#[derive(Debug, TypedBuilder)]
pub struct Request {
    pub headers: HashMap<String, String>,
    #[builder(setter(into))]
    pub method: String,
    #[builder(setter(into))]
    pub url: String,
    #[builder(default, setter(strip_option))]
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct Response {
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

    pub fn run(&self, name: &str, req: Request) -> Result<Response> {
        self.ctx.with(|ctx| {
            let global = ctx.globals();
            let handlers: Object = global.get("handlers")?;
            let func: Function = handlers.get(name)?;
            let v: Promise = func.call((req,))?;
            Ok::<_, anyhow::Error>(v.finish()?)
        })
    }
}

impl<'js> IntoJs<'js> for Request {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let obj = Object::new(ctx.clone())?;

        obj.set("headers", self.headers.into_js(ctx))?;
        obj.set("method", self.method.into_js(ctx))?;
        obj.set("url", self.url.into_js(ctx))?;
        obj.set("body", self.body.into_js(ctx))?;

        Ok(obj.into())
    }
}

impl<'js> FromJs<'js> for Response {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        let status: u16 = obj.get("status")?;
        let headers: HashMap<String, String> = obj.get("headers")?;
        let body: Option<String> = obj.get("body")?;

        Ok(Response {
            status,
            headers,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_worker_should_run() {
        let code = r#"
        (function(){async function hello(req){return{status:200,headers:{"content-type":"application/json"},body: JSON.stringify(req)};}return{hello:hello};})();"#;

        let req = Request::builder()
            .method("GET")
            .url("http://localhost:8080")
            .headers(HashMap::new())
            .build();
        let worker = JsWorker::try_new(code).unwrap();
        let ret = worker.run("hello", req).unwrap();
        assert_eq!(ret.status, 200);
    }
}
