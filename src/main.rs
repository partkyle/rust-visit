extern crate hyper;
extern crate futures;
extern crate redis;
extern crate hostname;

extern crate serde_json;

#[macro_use] extern crate serde_derive;

use std::env;

use redis::Commands;
use redis::RedisResult;

use futures::future::Future;

use hostname::get_hostname;

use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};
use hyper::{Method};

#[derive(Serialize)]
struct Healthcheck<'a> {
    version: &'a str,
    hostname: &'a str,
}

struct HelloWorld<'a> {
    redis_host: &'a str,
    hostname: &'a str,
}

impl<'a> HelloWorld<'a> {
    fn update_count(&self) -> RedisResult<u64> {
        let redis_client = redis::Client::open(self.redis_host).unwrap();
        let con = redis_client.get_connection().unwrap();

        con.incr("rust.visit.count", 1)
    }
}

impl<'a> Service for HelloWorld<'a> {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;

    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let content = match (req.method(), req.path()) {
            (&Method::Get, "/") => {
                let count = match self.update_count() {
                    Ok(count) => { count }
                    _ => { 0 }
                };
                format!("The current visit count is {} on {}.\n", count, self.hostname)
            }
            (&Method::Get, "/healthcheck") => {
                let healthcheck = Healthcheck{version: VERSION, hostname: self.hostname};
                serde_json::to_string_pretty(&healthcheck).unwrap()
            },

            _ => {
                let response = Response::new()
                                            .with_status(hyper::StatusCode::NotFound);
                return Box::new(futures::future::ok(response));
            }
        };

        let response = Response::new()
                                .with_header(ContentLength(content.len() as u64))
                                .with_body(content);
        Box::new(futures::future::ok(response))
    }
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let hostname = get_hostname().unwrap();

    let addr = match env::var("VISIT_ADDR") {
        Ok(val) => val,
        _ => "127.0.0.1:3000".to_string()
    };

    let redis_host = match env::var("VISIT_REDIS") {
        Ok(val) => val,
        _ => "redis://127.0.0.1/".to_string()
    };

    let addr = addr.parse().unwrap();

    let server = Http::new().bind(&addr, move || Ok(HelloWorld{hostname: &hostname, redis_host: &redis_host[..]})).unwrap();
    println!("running on {:?}", addr);

    server.run().unwrap();
}
