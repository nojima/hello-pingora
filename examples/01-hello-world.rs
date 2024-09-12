// "Hello, world!" を返す HTTP サーバー。
// 3000 番ポートで待ち受ける。

use async_trait::async_trait;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use http::Response;
use pingora::apps::http_app::ServeHttp;
use pingora::protocols::http::ServerSession;
use pingora::server::Server;
use pingora::services::listening::Service;

struct HelloApp;

#[async_trait]
impl ServeHttp for HelloApp {
    async fn response(&self, _server_session: &mut ServerSession) -> Response<Vec<u8>> {
        let message = b"Hello, world!\r\n".to_vec();
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

    let mut hello_service = Service::new("hello app".to_owned(), HelloApp);
    hello_service.add_tcp("[::]:3000");
    server.add_service(hello_service);

    server.run_forever();
}
