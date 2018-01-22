extern crate msgpack_rpc;
extern crate rmp as msgpack;
extern crate rmpv;

use std::thread;

use rmpv::Value;
use msgpack_rpc::*;

#[derive(Clone, Default)]
struct EchoServer;

impl Dispatch for EchoServer {
    fn dispatch(&mut self, method: &str, args: Vec<Value>) -> Result<Value, Value> {
        match method {
            "echo" => Ok(Value::Array(args.to_owned())),
            _ => Err(Value::from("Invalid method name.".to_owned())),
        }
    }
}

#[test]
fn echo() {
    let server = Server::bind("localhost:0").unwrap();
    let mut client = Client::connect_socket(server.local_addr().unwrap());

    thread::spawn(move || {
        server.handle(EchoServer);
    });

    let result = client.call("echo", vec![Value::from("Hello, world!".to_owned())]);
    assert_eq!(Value::Array(vec![Value::from("Hello, world!".to_owned())]),
               result.unwrap());
}

#[test]
fn invalid_method_name() {
    let server = Server::bind("localhost:0").unwrap();
    let mut client = Client::connect_socket(server.local_addr().unwrap());

    thread::spawn(move || {
        server.handle(EchoServer);
    });

    let result = client.call("bad_method", vec![]);
    assert_eq!(Value::from("Invalid method name.".to_owned()),
               result.unwrap_err());
}
