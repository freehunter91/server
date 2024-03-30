use std::net::TcpListener;
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
                Ok(mut stream, _) => {
                    let mut buffer = [0; 2024];
                    match stream.read(&mut buffer) {
                        Ok(_) => {
                            println!("Received a request: {}", String::from_utf8_lossy(&buffer));
                        }
                        Err(e) => println!("Failed to read from connectin: {}", e)
                    }
                },
                Err(e) => println!("연결 실패 : {} ", e),
                
            }


        }

    }
}
