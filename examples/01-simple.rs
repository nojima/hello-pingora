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
        let peer = HttpPeer::new("127.0.0.1:9000", false, "".to_string());
        Ok(Box::new(peer))
    }
}

fn main() {
    env_logger::init();

    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    let mut lb = http_proxy_service(&my_server.configuration, LB);
    lb.add_tcp("0.0.0.0:8000");
    my_server.add_service(lb);

    my_server.run_forever();
}
