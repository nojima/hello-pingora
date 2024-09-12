// 毎秒 upstreams.txt を読み込んで、その内容を使ってロードバランサーを更新する。
// upstreams.txt には複数の upstream のアドレスを記述でき、このLBはそれらに対してラウンドロビンでプロキシする。
// upstream には毎秒ヘルスチェックを行い、ダウンしているものは選択されない。
// 8000 番ポートで待ち受ける。

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use pingora::lb::discovery::ServiceDiscovery;
use pingora::lb::{Backend, Backends};
use pingora::prelude::*;
use pingora::protocols::l4::socket::SocketAddr;

struct LB {
    load_balancer: Arc<LoadBalancer<RoundRobin>>,
}

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {
        ()
    }

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self
            .load_balancer
            .select(b"", 256)
            .ok_or_else(|| Error::explain(ErrorType::HTTPStatus(503), "no available upstreams"))?;
        let peer = HttpPeer::new(upstream.addr, false, "".to_string());
        Ok(Box::new(peer))
    }
}

struct SD;

#[async_trait]
impl ServiceDiscovery for SD {
    async fn discover(&self) -> Result<(BTreeSet<Backend>, HashMap<u64, bool>)> {
        log::info!("SD::discover called");
        let addrs = read_upstreams_from_file()?;
        let backends = addrs
            .into_iter()
            .map(|addr| Backend { addr, weight: 1 })
            .collect::<BTreeSet<_>>();
        Ok((backends, HashMap::new()))
    }
}

fn read_upstreams_from_file() -> Result<Vec<SocketAddr>> {
    let contents = std::fs::read_to_string("upstreams.txt")
        .map_err(|e| Error::because(ErrorType::InternalError, "reading upstreams file", e))?;
    let addrs = contents
        .lines()
        .filter(|line| !line.trim().is_empty()) // remove empty lines
        .map(|line| {
            line.trim().parse::<SocketAddr>().map_err(|e| {
                Error::because(ErrorType::InternalError, "parsing upstream address", e)
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(addrs)
}

fn main() {
    env_logger::init();

    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    let backends = Backends::new(Box::new(SD));
    let mut load_balancer = LoadBalancer::from_backends(backends);
    load_balancer.set_health_check(TcpHealthCheck::new());
    load_balancer.health_check_frequency = Some(Duration::from_secs(1));
    load_balancer.update_frequency = Some(Duration::from_secs(1));
    let background = background_service("load balancer", load_balancer);

    let mut lb = http_proxy_service(
        &my_server.configuration,
        LB {
            load_balancer: background.task(),
        },
    );
    lb.add_tcp("[::]:8000");
    my_server.add_service(lb);
    my_server.add_service(background);

    my_server.run_forever();
}
