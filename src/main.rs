mod backend;
mod components;
mod models;
mod notica_component;
mod utils;

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
use components::notes_history::NotesHistoryPage;
use components::settings::SettingsPage;
use components::toast::ToastProvider;
use notica_component::NoticaApp;

#[cfg(feature = "server")]
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();
    use axum::{routing::get, Router};
    use axum_prometheus::PrometheusMetricLayerBuilder;

    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_default_metrics()
        .build_pair();

    let metrics = Router::new().route("/metrics", get(|| async move { metric_handle.render() }));

    let metrics_ip = std::env::var("HD_METRICS_IP").unwrap_or_else(|_| "0.0.0.0".to_string());
    let metrics_port = std::env::var("HD_METRICS_PORT").unwrap_or_else(|_| "9090".to_string());

    let metrics_listener =
        tokio::net::TcpListener::bind(format!("{metrics_ip}:{metrics_port}")).await?;

    let router = dioxus::server::router(app).layer(prometheus_layer);

    let ip = std::env::var("IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener = tokio::net::TcpListener::bind(format!("{ip}:{port}")).await?;

    // triggers both app router and metrics router
    tokio::try_join!(
        axum::serve(listener, router),
        axum::serve(metrics_listener, metrics)
    )?;

    Ok(())
}

#[cfg(not(feature = "server"))]
fn main() {
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
    #[route("/history")]
    NotesHistoryView,
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

#[component]
fn NotesHistoryView() -> Element {
    rsx! {
        div { style: "min-height: 100vh; background-color: #f3f4f6;",
            div { style: "max-width: 720px; margin: 0 auto; padding: 2rem 1rem;",
                NotesHistoryPage {}
            }
        }
    }
}
