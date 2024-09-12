// リクエストを 127.0.0.1:3000 に転送するロードバランサー。
// 8000 番ポートで待ち受ける。

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
        let peer = HttpPeer::new("127.0.0.1:3000", false, "".to_string());
        Ok(Box::new(peer))
    }
}

fn main() -> pingora::Result<()> {
    env_logger::init();

    let mut my_server = Server::new(None)?;
    my_server.bootstrap();

    let mut lb = http_proxy_service(&my_server.configuration, LB);
    lb.add_tcp("[::]:8000");
    my_server.add_service(lb);

    my_server.run_forever();
}
