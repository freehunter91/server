use crate::http::{Request, Response, StatusCode};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::io::{Read, Write};
use std::net::TcpListener;

use crate::http::request;
pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
    pub fn run(self) {
        println!("Listening on {}", self.addr);

        let listener = TcpListener::bind(&self.addr).unwrap();

        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buffer = [0; 2024];
                    match stream.read(&mut buffer) {
                        Ok(_) => {
                            println!("Received a request: {}", String::from_utf8_lossy(&buffer));

                            let respone = match Request::try_from(&buffer[..]) {
                                Ok(request) => {
                                    dbg!(request);
                                    Response::new(
                                        StatusCode::Ok,
                                        Some("<h1>IT WORK!</h1>".to_string()),
                                    );
                                    respone.send(&mut stream);
                                }
                                Err(e) => {
                                    println!("Failed to parse a request: {}", e);
                                    Response::new(StatusCode::BadRequest, None).send(&mut stream);
                                }
                            };
                            if let Err(e) = respone.send(&mut stream) {
                                println!("Failed to send response: {}", e);
                            }
                        }
                        Err(e) => println!("Failed to read from connectin: {}", e),
                    }
                }
                Err(e) => println!("연결 실패 : {} ", e),
            }
        }
    }
}
