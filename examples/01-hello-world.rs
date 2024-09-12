// "Hello, world!" を返す HTTP サーバー。
// 3000 番ポートで待ち受ける。

use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use http::Response;
use pingora::apps::http_app::ServeHttp;
use pingora::protocols::http::ServerSession;
use pingora::server::Server;
use pingora::services::listening::Service;

struct HelloApp {
    // アクセスカウンター
    counter: AtomicU64,
}

#[async_trait]
impl ServeHttp for HelloApp {
    async fn response(&self, _server_session: &mut ServerSession) -> Response<Vec<u8>> {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);
        let message = format!("Hello, world!\r\nあなたは {n} 人目の訪問者です！\r\n").into_bytes();
        Response::builder()
            .status(200)
            .header(CONTENT_TYPE, "text/plain")
            .header(CONTENT_LENGTH, message.len())
            .body(message)
            .unwrap()
    }
}

fn main() -> pingora::Result<()> {
    env_logger::init();

    let mut server = Server::new(None)?;
    server.bootstrap();

    let hello_app = HelloApp {
        counter: AtomicU64::new(1),
    };
    let mut hello_service = Service::new("hello app".to_owned(), hello_app);
    hello_service.add_tcp("[::]:3000");
    server.add_service(hello_service);

    server.run_forever();
}
