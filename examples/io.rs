extern crate futures;
extern crate gotham;
extern crate gotham_middleware_threadpool;
extern crate hyper;
extern crate mime;

use std::fs::File;
use std::io::prelude::*;

use futures::future::Future;
use gotham::handler::HandlerFuture;
use gotham::http::response::create_response;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::{FromState, State};
use gotham_middleware_threadpool::{ThreadPoolMiddleware, ThreadPoolMiddlewareData};
use hyper::StatusCode;

pub fn say_hello(state: State) -> Box<HandlerFuture> {
    let f = {
        let mwd: &ThreadPoolMiddlewareData = ThreadPoolMiddlewareData::borrow_from(&state);

        mwd.spawn_fn(move || {
            let mut file = File::open("Cargo.toml")?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            Ok::<String, std::io::Error>(contents)
        })
    }.and_then(|text: String| {
        let res = create_response(
            &state,
            StatusCode::Ok,
            Some((text.into_bytes(), mime::TEXT_PLAIN)),
        );

        Ok((state, res))
    })
        .map_err(|_| unimplemented!());

    Box::new(f)
}

fn router() -> Router {
    let (chain, pipelines) = single_pipeline(
        new_pipeline()
            .add(ThreadPoolMiddleware::with_num_cpus())
            .build(),
    );

    build_router(chain, pipelines, |route| {
        route.get("/").to(say_hello);
    })
}

pub fn main() {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router())
}
