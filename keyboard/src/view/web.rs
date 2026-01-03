use crate::events::{Event, ToDevice};
use crate::view::hexboard_view::HexBoardView;
use crate::view::launchpad_view::LaunchpadView;
use crate::view::state::{AppState, LockedState};
use crate::{DeviceType, events};
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
            // We only serve js assets, so hard-code the content type. There is a `mime_guess`
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

async fn empty() -> impl IntoResponse {
    StatusCode::OK
}

async fn launchpad_view(State(lock): State<LockedState>) -> impl IntoResponse {
    Html(LaunchpadView::generate_view(lock).await)
}

async fn launchpad_board(State(lock): State<LockedState>) -> impl IntoResponse {
    Html(LaunchpadView::generate_board(lock).await)
}

async fn hexboard_view(State(lock): State<LockedState>) -> impl IntoResponse {
    Html(HexBoardView::generate_view(lock).await)
}

async fn hexboard_board(State(lock): State<LockedState>) -> impl IntoResponse {
    Html(HexBoardView::generate_board(lock).await)
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

pub async fn http_view(
    events_tx: events::WeakSender,
    events_rx: events::Receiver,
    port: u16,
    view: DeviceType,
) {
    let state: LockedState = AppState::new_locked(events_tx);
    let app = Router::new();
    let mut app = app
        .route("/sse", get(sse_handler))
        .route("/assets/{*path}", get(static_asset));
    // Axum is limited in how much you can use generics for handlers, so rather than using traits,
    // we just have to explicitly name the various viewers.
    match view {
        DeviceType::Empty => app = app.route("/", get(empty)),
        DeviceType::Launchpad => app = app.route("/", get(launchpad_view)),
        DeviceType::HexBoard => app = app.route("/", get(hexboard_view)),
    }
    match view {
        DeviceType::Empty => app = app.route("/board", get(empty)),
        DeviceType::Launchpad => app = app.route("/board", get(launchpad_board)),
        DeviceType::HexBoard => app = app.route("/board", get(hexboard_board)),
    }

    let app = app.with_state(state.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Web View running at http://{addr}");
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
            Event::ToDevice(td) => match td {
                ToDevice::Light(e) => state.write().await.handle_light_event(&e),
            },
            Event::SelectLayout(e) => state.write().await.handle_select_layout(e).await,
            Event::SetLayoutNames(e) => state.write().await.handle_layout_names(e).await,
            Event::Reset => state.write().await.handle_reset().await,
            #[cfg(test)]
            Event::TestWeb(test_tx) => {
                test_tx
                    .send(state.read().await.get_state_view().clone())
                    .await
                    .unwrap();
            }
            _ => {}
        }
    }
}

async fn graceful_shutdown(rx: oneshot::Receiver<()>, state: LockedState) {
    _ = rx.await;
    println!("received shutdown signal");
    state.write().await.shutdown();
}
