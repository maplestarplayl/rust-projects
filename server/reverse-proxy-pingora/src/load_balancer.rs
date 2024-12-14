use pingora_load_balancing::selection::RoundRobin;
use pingora_load_balancing::LoadBalancer;
use std::sync::Arc;
use pingora_proxy::{Session,ProxyHttp};
use pingora::prelude::*;
use async_trait::async_trait;
pub struct LB(pub Arc<LoadBalancer<RoundRobin>>);

impl LB {
    pub fn get_request_appid(&self, session: &mut Session) -> Option<String> {
        match session.req_header().headers.get("appid")
            .map(|v| v.to_str())
        {   
            Some(Ok(appid)) => Some(appid.to_string()),
            _ => None,
        }
    }
}

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {
        ()
    }

    async fn upstream_peer(&self, _session: &mut Session,_ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self.0.select(b"", 256).unwrap();
        println!("upstream: {:?}", upstream);
        // 注意http和https的区别
        let peer = Box::new(HttpPeer::new(upstream,false,"one.one.one.one".to_string()));
        Ok(peer)
    }
    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        req: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        req.insert_header("Host", "one.one.one.one".to_string()).unwrap();
        Ok(())
    }
}
