use crate::events;
use crate::events::{Event, KeyEvent};
use crate::view::content::App;
use crate::view::state::{AppState, LockedState};
use askama::Template;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::Sse;
use axum::routing::post;
use axum::{
    Form, Router,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use rust_embed::RustEmbed;
use serde::Deserialize;
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

#[derive(Deserialize, Debug)]
struct KeyInput {
    key: u8,
    velocity: u8,
}

async fn view(State(lock): State<LockedState>) -> impl IntoResponse {
    let s = lock.read().await;
    Html(
        App::new(s.get_cells(), s.get_state_view())
            .render()
            .unwrap(),
    )
}

async fn key(State(lock): State<LockedState>, data: Form<KeyInput>) -> impl IntoResponse {
    let Some(tx) = lock.read().await.get_events_tx() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    if let Err(e) = tx.send(Event::Key(KeyEvent {
        key: data.key,
        velocity: data.velocity,
    })) {
        log::error!("web server: error sending key event: {e}");
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    StatusCode::ACCEPTED.into_response()
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

pub async fn http_view(events_tx: events::WeakSender, events_rx: events::Receiver, port: u16) {
    let state: LockedState = AppState::new_locked(events_tx);
    let app = Router::new()
        .route("/sse", get(sse_handler))
        .route("/assets/{*path}", get(static_asset))
        .route("/key", post(key))
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
            Event::SelectLayout(e) => state.write().await.handle_select_layout(e).await,
            Event::Reset => state.write().await.handle_reset().await,
            _ => {}
        }
    }
}

async fn graceful_shutdown(rx: oneshot::Receiver<()>, state: LockedState) {
    _ = rx.await;
    state.write().await.shutdown();
    log::info!("received shutdown signal");
}
