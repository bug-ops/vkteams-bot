//! Make health reporter for gRPC
//! https://github.com/grpc/grpc/blob/master/doc/health-checking.md
//!
//! [supported in Kubernetes by default](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/#define-a-grpc-liveness-probe)
//! ## Example
//! ```
//! extern crate tonic_health;
//! extern crate tonic;
//! extern crate anyhow;
//! use crate::vkteams_bot::bot::net::shutdown_signal;
//! 
//! const DEFAULT_TCP_PORT: &str = "VKTEAMS_BOT_HTTP_PORT";
//! pub async fn run_probe_app() -> anyhow::Result<()> {
//!     // Create gRPC health reporter and service
//!     let (_, health_service) = tonic_health::server::health_reporter();
//!     // Get the port from the environment variable or use the default port 50555
//!     let tcp_port = std::env::var(DEFAULT_TCP_PORT).unwrap_or_else(|_| "50555".to_string());
//!     // Start gRPC server
//!     tonic::transport::Server::builder()
//!         .add_service(health_service)
//!         .serve_with_shutdown(
//!             format!("[::1]:{tcp_port}").parse().unwrap(),
//!             shutdown_signal(),
//!         )
//!         .await?;
//!     Ok(())
//! }
//! ```
use axum::Router;
/// Inherit Router with gRPC probe
pub trait GRPCRouter<S> {
    fn route_grpc_probe(self) -> Self;
}
/// Implement GRPCRouter for Router
impl<S> GRPCRouter<S> for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn route_grpc_probe(self) -> Self {
        let (_, health_service) = tonic_health::server::health_reporter();
        self.route_service("/grpc.health.v1.Health/Check", health_service)
    }
}
