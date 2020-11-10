use http_service as hs;

mod http_service;
mod middleware;

fn main() {
    hs::main::launch();
}