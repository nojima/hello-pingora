// リクエストが来るたびに upstream.txt からアドレスを読み込んでそこにプロキシする。
// 8000 番ポートで待ち受ける。

use std::net::SocketAddr;

use async_trait::async_trait;
use pingora::prelude::*;

struct LB;

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {
        ()
    }

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let addr = read_upstream_from_file()?;
        let peer = HttpPeer::new(addr, false, "".to_string());
        Ok(Box::new(peer))
    }
}

fn read_upstream_from_file() -> Result<SocketAddr> {
    let content = std::fs::read_to_string("upstream.txt")
        .map_err(|e| Error::because(ErrorType::HTTPStatus(500), "reading upstream file", e))?;
    let addr = content
        .trim()
        .parse::<SocketAddr>()
        .map_err(|e| Error::because(ErrorType::HTTPStatus(500), "parsing upstream address", e))?;
    Ok(addr)
}

fn main() {
    env_logger::init();

    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    let mut lb = http_proxy_service(&my_server.configuration, LB);
    lb.add_tcp("[::]:8000");
    my_server.add_service(lb);

    my_server.run_forever();
}
