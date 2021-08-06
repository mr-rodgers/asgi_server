mod providers;
mod scope_builder;
mod scope_provider;

use std::net::SocketAddr;

use crate::asgi_scope;
use scope_builder::HttpScopeBuilder;

pub fn build(
    parts: http::request::Parts,
    client_addr: SocketAddr,
    server_addr: SocketAddr,
) -> impl asgi_scope::ScopeProvider {
    asgi_scope::ScopeBuilder::new()
        .add_provider(asgi_scope::providers::Type::HTTP)
        .add_provider(providers::HttpAddress::ClientSocket(client_addr))
        .add_provider(providers::HttpAddress::ServerSocket(server_addr))
        .add_provider(
            HttpScopeBuilder::new(parts)
                .add_provider(providers::HttpVersion {})
                .add_provider(providers::HttpScheme {})
                .add_provider(providers::HttpMethod {})
                .add_provider(providers::HttpPath {})
                .add_provider(providers::HttpQueryString {})
                .add_provider(providers::HttpHeaders {}),
        )
}
