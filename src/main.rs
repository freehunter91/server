// 매일 30분씩 강의 듣으면서 따라하도록하자 한번에 보기에는 무리가 있는데, 여유 시간 있을 때는 장시간 투자할 수 있을 때하도록 하자.
use http::Method;
use http::Request;
use server::Server;

mod http;
mod server;

fn main() {
    let server = Server::new("127.0.0.1:8080".to_string());
    server.run();
}
