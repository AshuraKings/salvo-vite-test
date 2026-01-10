use salvo::{conn::tcp::TcpAcceptor, prelude::*, server::ServerHandle};

// Handler for English greeting
#[handler]
async fn hello() -> &'static str {
    "Hello World"
}

// Handler for Chinese greeting
#[handler]
async fn hello_zh() -> Result<&'static str, ()> {
    Ok("你好，世界！")
}

async fn router() -> Router {
    Router::new()
        .get(hello)
        .push(Router::with_path("你好").get(hello_zh))
}

async fn server() -> Server<TcpAcceptor> {
    let acceptor = TcpListener::new("0.0.0.0:8698").bind().await;
    Server::new(acceptor)
}

async fn gracefull_shutdown(handle: ServerHandle) -> anyhow::Result<()> {
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
    tokio::select! {
        _=sigterm.recv() =>{
            println!("Received SIGTERM. Initiating graceful shutdown.");
        },
        _=sigint.recv() =>{
            println!("Received SIGINT (Ctrl+C). Initiating graceful shutdown.");
        },
    }
    handle.stop_graceful(None);
    Ok(())
}

#[tokio::main]
async fn main() {
    // Initialize logging subsystem
    tracing_subscriber::fmt().init();

    // Bind server to port 8698
    let server = server().await;
    let handle = server.handle();
    tokio::spawn(async move {
        if let Err(e) = gracefull_shutdown(handle).await {
            tracing::error!("{e:?}");
        }
    });

    // Create router with two endpoints:
    // - / (root path) returns English greeting
    // - /你好 returns Chinese greeting
    let router = router().await;

    // Print router structure for debugging
    println!("{router:?}");

    // Start serving requests
    server.serve(router).await;
}
