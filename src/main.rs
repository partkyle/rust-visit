extern crate hyper;
extern crate futures;
extern crate redis;
extern crate hostname;

use std::env;

use redis::Commands;
use redis::RedisResult;

use futures::future::Future;

use hostname::get_hostname;

use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};

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

    fn call(&self, _req: Request) -> Self::Future {
        let count = match self.update_count() {
            Ok(count) => { count }
            _ => { 0 }
        };
        let phrase = format!("The current visit count is {} on {}.\n", count, self.hostname);
        let response = Response::new()
                                .with_header(ContentLength(phrase.len() as u64))
                                .with_body(phrase);
        Box::new(futures::future::ok(response))
    }
}

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
