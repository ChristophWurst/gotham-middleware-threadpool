extern crate futures;
extern crate futures_cpupool;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;

use std::io;
use std::panic::AssertUnwindSafe;

use futures::future::{Future, IntoFuture};
use gotham::handler::HandlerFuture;
use gotham::middleware::{Middleware, NewMiddleware};
use gotham::state::State;

use futures_cpupool::CpuPool;

pub struct ThreadPoolMiddleware {
    pool: AssertUnwindSafe<CpuPool>,
}

impl ThreadPoolMiddleware {
    pub fn new(pool: CpuPool) -> Self {
        ThreadPoolMiddleware {
            pool: AssertUnwindSafe(pool),
        }
    }

    pub fn with_num_cpus() -> Self {
        let pool = futures_cpupool::CpuPool::new_num_cpus();
        Self::new(pool)
    }

    pub fn with_size(size: usize) -> Self {
        let pool = futures_cpupool::CpuPool::new(size);
        Self::new(pool)
    }
}

impl Middleware for ThreadPoolMiddleware {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture> + 'static,
        Self: Sized,
    {
        state.put(ThreadPoolMiddlewareData::new(self.pool.clone()));

        chain(state)
    }
}

impl NewMiddleware for ThreadPoolMiddleware {
    type Instance = ThreadPoolMiddleware;

    fn new_middleware(&self) -> io::Result<Self::Instance> {
        Ok(ThreadPoolMiddleware {
            pool: AssertUnwindSafe(self.pool.clone()),
        })
    }
}

#[derive(StateData)]
pub struct ThreadPoolMiddlewareData {
    pool: CpuPool,
}

impl ThreadPoolMiddlewareData {
    pub fn new(pool: CpuPool) -> Self {
        ThreadPoolMiddlewareData { pool: pool }
    }

    pub fn spawn<F>(&self, f: F) -> Box<Future<Item = F::Item, Error = F::Error>>
    where
        F: Future + Send + 'static,
        F::Item: Send + 'static,
        F::Error: Send + 'static,
    {
        Box::new(self.pool.spawn(f))
    }

    pub fn spawn_fn<F, R>(&self, f: F) -> Box<Future<Item = R::Item, Error = R::Error>>
    where
        F: FnOnce() -> R + Send + 'static,
        R: IntoFuture + 'static,
        R::Future: Send + 'static,
        R::Item: Send + 'static,
        R::Error: Send + 'static,
    {
        Box::new(self.pool.spawn_fn(f))
    }
}
