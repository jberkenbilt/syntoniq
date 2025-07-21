use crate::events;
use crate::events::Event;
use crate::view::content::App;
use crate::view::state::LockedState;
use askama::Template;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::Sse;
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use std::sync::LazyLock;
use tokio::sync::{Mutex, oneshot};
use tokio::task;
use tokio_stream::StreamExt; // for .map
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

static SHUTDOWN: LazyLock<Mutex<Option<oneshot::Sender<()>>>> = LazyLock::new(Default::default);

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

async fn static_asset(Path(path): Path<String>) -> impl IntoResponse {
    match Assets::get(&path) {
        Some(content) => {
            // We only server js assets, so hard-code the content type. There is a `mime_guess`
            // crate that could help.
            (
                [(header::CONTENT_TYPE, "application/javascript")],
                content.data,
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn view(State(lock): State<LockedState>) -> impl IntoResponse {
    let s = lock.read().await;
    Html(App::new(s.get_cells()).render().unwrap())
}

async fn sse_handler(State(lock): State<LockedState>) -> impl IntoResponse {
    let rx = {
        let Some(tx) = lock.read().await.get_sse_tx() else {
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        };
        tx.subscribe()
    };

    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(event) => Some(Result::<_, BroadcastStreamRecvError>::Ok(event)),
        Err(BroadcastStreamRecvError::Lagged(count)) => {
            log::warn!("SSE client lagged (count={count})");
            None
        }
    });
    Sse::new(stream).into_response()
}

pub async fn http_view(events_rx: events::Receiver, port: u16) {
    let state: LockedState = Default::default();
    let app = Router::new()
        .route("/sse", get(sse_handler))
        .route("/assets/{*path}", get(static_asset))
        .route("/", get(view))
        .with_state(state.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    log::info!("View HTTP server listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let (tx, rx) = oneshot::channel();
    *SHUTDOWN.lock().await = Some(tx);
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        SHUTDOWN.lock().await.take();
    });
    let s2 = state.clone();
    task::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(graceful_shutdown(rx, s2))
            .await
            .expect("unable to start HTTP");
    });
    main_loop(state, events_rx).await;
}

async fn main_loop(state: LockedState, mut events_rx: events::Receiver) {
    while let Some(event) =
        events::receive_check_lag(&mut events_rx, Some("view http server")).await
    {
        match event {
            Event::Shutdown => drop(SHUTDOWN.lock().await.take()),
            Event::Light(e) => state.write().await.handle_light_event(e),
            _ => {}
        }
    }
}

async fn graceful_shutdown(rx: oneshot::Receiver<()>, state: LockedState) {
    _ = rx.await;
    state.write().await.shutdown();
    log::info!("received shutdown signal");
}
