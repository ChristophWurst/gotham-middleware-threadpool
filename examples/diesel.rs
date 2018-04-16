extern crate diesel;
extern crate futures;
extern crate gotham;
extern crate gotham_middleware_diesel;
extern crate gotham_middleware_threadpool;
extern crate hyper;
extern crate mime;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use futures::future::Future;
use gotham::handler::HandlerFuture;
use gotham::http::response::create_response;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::{FromState, State};
use gotham_middleware_threadpool::{ThreadPoolMiddleware, ThreadPoolMiddlewareData};
use gotham_middleware_diesel::{DieselMiddleware, state_data::connection};
use hyper::StatusCode;

pub fn say_hello(state: State) -> Box<HandlerFuture> {
    let f = {
        let mwd: &ThreadPoolMiddlewareData = ThreadPoolMiddlewareData::borrow_from(&state);
        let conn = connection::<SqliteConnection>(&state);

        mwd.spawn_fn(move || {
            conn.execute("SELECT 1").unwrap();
            Ok::<String, std::io::Error>("Hello, Diesel".to_owned())
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
    let dmw: DieselMiddleware<SqliteConnection> = DieselMiddleware::new(":memory:");

    let (chain, pipelines) = single_pipeline(
        new_pipeline()
            .add(dmw)
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
