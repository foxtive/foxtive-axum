//! Testing utilities for foxtive-axum.
//!
//! Provides [`TestServer`] for spinning up a lightweight HTTP server
//! in integration tests.

use axum::Router;
use foxtive::App;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

/// A lightweight test HTTP server for integration testing.
///
/// `TestServer` binds to a random available port on `127.0.0.1`,
/// runs the given router in the background, and provides the base URL
/// for making test requests.
///
/// # Example
/// ```rust,ignore
/// use axum::routing::get;
/// use axum::Router;
/// use foxtive_axum::testing::TestServer;
///
/// let router = Router::new().route("/health", get(|| async { "ok" }));
/// let server = TestServer::start(router).await;
///
/// let resp = reqwest::get(server.url("/health")).await.unwrap();
/// assert_eq!(resp.status(), 200);
///
/// server.shutdown();
/// ```
pub struct TestServer {
    addr: SocketAddr,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl TestServer {
    /// Start a test server with the given router.
    ///
    /// The server binds to `127.0.0.1:0` (random port) and runs
    /// in a background tokio task.
    pub async fn start(router: Router) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind test server");
        let addr = listener.local_addr().expect("Failed to get test server address");

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .ok();
        });

        // Give the server a moment to start accepting connections
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        Self { addr, shutdown_tx }
    }

    /// Start a test server with a router and foxtive App (with Extension layer).
    ///
    /// This injects `Arc<App>` as an Extension so handlers can extract it
    /// via `Extension(app): Extension<Arc<App>>`.
    pub async fn start_with_app(router: Router, app: Arc<App>) -> Self {
        let router = router.layer(axum::Extension(app));
        Self::start(router).await
    }

    /// Returns the full URL for a given path.
    ///
    /// # Example
    /// ```rust,ignore
    /// # use foxtive_axum::testing::TestServer;
    /// # use axum::Router;
    /// # let server = TestServer::start(Router::new()).await;
    /// let url = server.url("/api/users");
    /// assert!(url.starts_with("http://127.0.0.1:"));
    /// ```
    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }

    /// Returns the socket address the server is listening on.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Shut down the test server gracefully.
    pub fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
    }
}
