use axum::Extension;
use axum::Router;
use axum::routing::get;
use foxtive::App;
use foxtive::results::AppResult;
use foxtive::setup::trace::Tracing;
use foxtive_axum::http::HttpResult;
use foxtive_axum::http::response::ext::StructResponseExt;
use foxtive_axum::server::Server;
use socketioxide::extract::{Data, SocketRef};
use socketioxide::SocketIo;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Build the App (DI container) with tracing and env configuration
    let app = App::builder("SocketIO Example", "SOCKETIO")
        .environment(foxtive::Environment::Local)
        .tracing(Tracing::default())
        .build()
        .await?;

    let (svc, io) = SocketIo::new_svc();

    io.ns("/", on_connect);
    io.ns("/chat", on_chat_connect);

    let router = Router::new().route("/", get(root_handler));

    Server::new(app)
        .host("127.0.0.1")
        .port(3000)
        .router(router)
        .has_started_bootstrap(true) // Tracing already initialized above
        .nest_service("/socket.io", svc)
        .bootstrap(|app| async move {
            info!("Bootstrapping application: {}", app.app_name());
            Ok(())
        })
        .on_started(async {
            info!("Server started at http://127.0.0.1:3000");
            info!("Socket.IO endpoint available at /socket.io/");
        })
        .run()
        .await
}

async fn root_handler(Extension(_app): Extension<Arc<App>>) -> HttpResult {
    info!("Root handler called");
    "Socket.IO example - connect to /socket.io/".respond()
}

async fn on_connect(s: SocketRef) {
    info!("Socket connected: {}", s.id);

    s.on("message", async |s: SocketRef, Data(data): Data<String>| {
        info!("Received message: {}", data);
        let response = format!("Echo: {}", data);
        s.emit("message-back", &response).ok();
    });

    s.on_disconnect(async |s: SocketRef| {
        info!("Socket disconnected: {}", s.id);
    });
}

async fn on_chat_connect(s: SocketRef) {
    info!("Chat socket connected: {}", s.id);

    s.on("chat-message", async |s: SocketRef, Data(data): Data<String>| {
        info!("Chat message: {}", data);
        let response = format!("Chat echo: {}", data);
        s.emit("chat-response", &response).ok();
    });
}
