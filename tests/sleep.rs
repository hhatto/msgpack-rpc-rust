extern crate msgpack_rpc;
extern crate rmpv;

use std::thread;
use std::time::Duration;

use rmpv::Value;
use msgpack_rpc::*;

#[derive(Clone, Default)]
pub struct SleepServer;

impl Dispatch for SleepServer {
    fn dispatch(&mut self, method: &str, args: Vec<Value>) -> Result<Value, Value> {
        match method {
            "sleep" => {
                if let Value::Integer(secs) = *args.get(0).unwrap() {
                    let secs = secs.as_u64().expect("fail convert u63");
                    thread::sleep(Duration::from_secs(secs));
                    Ok(Value::from(secs))
                } else {
                    Err(Value::from(format!("Invalid parameters: {:?}", args)))
                }
            }
            _ => Err(Value::from(format!("Invalid method: {}", method))),
        }
    }
}

#[test]
/// Ensures that requests that finish before long running requests are returned.
fn async() {
    let server = Server::bind("localhost:0").unwrap();
    let mut client = Client::connect_socket(server.local_addr().unwrap());

    thread::spawn(move || server.handle(SleepServer));

    // Sleep for a very long time.
    client.async_call("sleep", vec![Value::from(10_000)]);

    let short_result = client.async_call("sleep", vec![Value::from(1)]);

    assert!(match short_result.recv().unwrap() {
        Ok(Value::Integer(_)) => true,
        _ => false,
    });
}

#[test]
fn sleep() {
    let server = Server::bind("localhost:0").unwrap();
    let mut client = Client::connect_socket(server.local_addr().unwrap());

    thread::spawn(move || server.handle(SleepServer));

    let long_result = client.async_call("sleep", vec![Value::from(2)]);
    let short_result = client.async_call("sleep", vec![Value::from(1)]);

    assert!(match short_result.recv().unwrap() {
        Ok(Value::Integer(_)) => true,
        _ => false,
    });

    assert!(match long_result.recv().unwrap() {
        Ok(Value::Integer(_)) => true,
        _ => false,
    });
}
