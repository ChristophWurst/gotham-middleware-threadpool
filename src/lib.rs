extern crate futures_cpupool;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;

use std::io;
use std::panic::AssertUnwindSafe;

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
}

impl ThreadPoolMiddlewareData {
    pub fn get_pool(&self) -> CpuPool {
        self.pool.clone()
    }
}
