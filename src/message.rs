use std::io;
use std::io::prelude::*;

use rmpv;
use rmpv::Value;

#[derive(PartialEq, Clone, Debug)]
pub struct Request {
    pub id: u32,
    pub method: String,
    pub params: Vec<Value>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Response {
    pub id: u32,
    pub result: Result<Value, Value>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Notification {
    pub method: String,
    pub params: Vec<Value>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Message {
    Request(Request),
    Response(Response),
    Notification(Notification),
}

impl Message {
    pub fn msgtype(&self) -> i32 {
        use self::Message::*;

        match *self {
            Request(..) => 0,
            Response(..) => 1,
            Notification(..) => 2,
        }
    }

    pub fn unpack<R>(reader: &mut R) -> io::Result<Message>
        where R: Read
    {
        let value = rmpv::decode::read_value(reader).expect("Could not read value from transport");

        let array = match value {
            Value::Array(array) => array,
            _ => panic!("Invalid msgpack-rpc message received: {:?}", value),
        };

        let msg_type = match *array.get(0).unwrap() {
            Value::Integer(msg_type) => msg_type,
            _ => panic!(),
        };

        let message = match msg_type.as_i64() {
            Some(0) => {
                let id = if let Value::Integer(ref id) = *array.get(1).unwrap() {
                    id.as_u64().expect("fail convert u64")
                } else {
                    panic!();
                };

                let method = if let Value::String(ref method) = *array.get(2).unwrap() {
                    method
                } else {
                    panic!();
                };

                let params = if let Value::Array(ref params) = *array.get(3).unwrap() {
                    params
                } else {
                    panic!();
                };

                Message::Request(Request {
                    id: id as u32,
                    method: method.to_owned().into_str().expect("fail convert str"),
                    params: params.to_owned(),
                })
            }
            Some(1) => {
                let id = if let Value::Integer(ref id) = *array.get(1).unwrap() {
                    id.as_u64().expect("fail convert u64")
                } else {
                    panic!();
                };

                let err = array.get(2).unwrap().to_owned();
                let rpc_result = array.get(3).unwrap().to_owned();

                let result = match err {
                    Value::Nil => Ok(rpc_result),
                    _ => Err(err),
                };

                Message::Response(Response {
                    id: id as u32,
                    result: result,
                })
            }
            Some(2) => {
                let method = if let Value::String(ref method) = *array.get(1).unwrap() {
                    method
                } else {
                    panic!();
                };

                let params = if let Value::Array(ref params) = *array.get(2).unwrap() {
                    params
                } else {
                    panic!();
                };

                Message::Notification(Notification {
                    method: method.to_owned().into_str().expect("fail convert str"),
                    params: params.to_owned(),
                })
            }
            _ => unimplemented!(),
        };
        Ok(message)
    }

    pub fn pack(&self) -> Vec<u8> {
        let value = match *self {
            Message::Request(Request { id, ref method, ref params }) => {
                Value::Array(vec![
                    Value::from(self.msgtype() as u64),
                    Value::from(id),
                    Value::from(method.to_owned()),
                    Value::Array(params.to_owned()),
                ])
            }
            Message::Response(Response { id, ref result }) => {
                let (error, result) = match *result {
                    Ok(ref result) => (Value::Nil, result.to_owned()),
                    Err(ref err) => (err.to_owned(), Value::Nil),
                };

                Value::Array(vec![
                    Value::from(self.msgtype() as u64),
                    Value::from(id as u64),
                    error,
                    result,
                ])
            }
            Message::Notification(Notification { ref method, ref params }) => {
                Value::Array(vec![
                    Value::from(self.msgtype() as u64),
                    Value::from(method.to_owned()),
                    Value::Array(params.to_owned()),
                ])
            }
        };
        let mut bytes = vec![];
        rmpv::encode::write_value(&mut bytes, &value).unwrap();
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use rmpv::Value;

    #[test]
    fn pack_unpack_request() {
        let request = Message::Request(Request {
            id: 0,
            method: "echo".to_owned(),
            params: vec![Value::from("hello world!".to_owned())],
        });

        let bytes = request.pack();
        let mut cursor = Cursor::new(&bytes);
        let unpacked_request = Message::unpack(&mut cursor).unwrap();

        assert_eq!(request, unpacked_request);
    }

    #[test]
    fn pack_unpack_response() {
        let response = Message::Response(Response {
            id: 0,
            result: Ok(Value::from("test".to_owned())),
        });

        let bytes = response.pack();
        let mut cursor = Cursor::new(&bytes);
        let unpacked_response = Message::unpack(&mut cursor).unwrap();

        assert_eq!(response, unpacked_response);
    }

    #[test]
    fn pack_unpack_notification() {
        let notification = Message::Notification(Notification {
            method: "ping".to_owned(),
            params: vec![Value::from("hi".to_owned())],
        });

        let bytes = notification.pack();
        let mut cursor = Cursor::new(&bytes);
        let unpacked_notification = Message::unpack(&mut cursor).unwrap();

        assert_eq!(notification, unpacked_notification);
    }
}
