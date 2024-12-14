use pingora::prelude::*;
use pingora_load_balancing::LoadBalancer;
use std::sync::Arc;
mod load_balancer;
use load_balancer::LB;
fn main() {
    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    let upstreams = LoadBalancer::try_from_iter(["1.1.1.1:443","1.0.0.1:443"]).unwrap();
    
    let mut lb = http_proxy_service(&my_server.configuration, LB(Arc::new(upstreams)));
    lb.add_tcp("0.0.0.0:8080");
    my_server.add_service(lb);
    my_server.run_forever();
}
