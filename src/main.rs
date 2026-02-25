mod backend;
mod components;
mod models;
mod notica_component;

#[cfg(feature = "server")]
use std::net::{IpAddr, SocketAddr};

#[cfg(feature = "server")]
use axum::extract::ConnectInfo;

#[cfg(feature = "server")]
use axum::middleware::Next;
#[cfg(feature = "server")]
use axum::response::IntoResponse;
use dioxus::prelude::*;

use components::about::AboutPage;
use components::kid_history::KidHistoryPage;
use components::settings::SettingsPage;
use components::toast::ToastProvider;
use notica_component::NoticaApp;

#[cfg(feature = "server")]
fn is_flyio_internal(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V6(v6) => {
            let b = v6.octets();
            b[0] == 0xfd && b[1] == 0xaa
        }
        IpAddr::V4(v4) => v4.is_loopback(),
    }
}

#[cfg(feature = "server")]
async fn requires_fly_io_source(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    if is_flyio_internal(addr.ip()) {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        use axum::{middleware, routing::get};
        use axum_prometheus::PrometheusMetricLayerBuilder;

        let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
            .with_default_metrics()
            .build_pair();

        let router = dioxus::server::router(app)
            .route(
                "/metrics",
                get(move || async move { metric_handle.render() })
                    .layer(middleware::from_fn(requires_fly_io_source)),
            )
            .layer(prometheus_layer);

        Ok(router)
    });

    #[cfg(not(feature = "server"))]
    dioxus::launch(app);
}

fn app() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        document::Stylesheet { href: asset!("/assets/dx-components-theme.css") }
        ToastProvider { Router::<Route> {} }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Routable)]
pub enum Route {
    #[route("/")]
    MainView,
    #[route("/settings")]
    SettingsView,
    #[route("/about")]
    AboutView,
    #[route("/kid/:id")]
    KidHistory { id: u32 },
}

#[component]
fn MainView() -> Element {
    rsx! {
        div { style: "min-height: 100vh; background-color: #f3f4f6;",
            div { style: "max-width: 520px; margin: 0 auto; padding: 2rem 1rem;", NoticaApp {} }
        }
    }
}

#[component]
fn SettingsView() -> Element {
    rsx! {
        div { style: "min-height: 100vh; background-color: #f3f4f6;",
            div { style: "max-width: 520px; margin: 0 auto; padding: 2rem 1rem;", SettingsPage {} }
        }
    }
}

#[component]
fn AboutView() -> Element {
    rsx! {
        div { style: "min-height: 100vh; background-color: #f3f4f6;",
            div { style: "max-width: 520px; margin: 0 auto; padding: 2rem 1rem;", AboutPage {} }
        }
    }
}

#[component]
fn KidHistory(id: u32) -> Element {
    rsx! {
        div { style: "min-height: 100vh; background-color: #f3f4f6;",
            div { style: "max-width: 520px; margin: 0 auto; padding: 2rem 1rem;",
                KidHistoryPage { kid_id: id }
            }
        }
    }
}
