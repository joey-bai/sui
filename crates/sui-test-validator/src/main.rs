// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use clap::Parser;
use http::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use sui_cluster_test::{
    cluster::{Cluster, LocalNewCluster},
    config::{ClusterTestOpt, Env},
    faucet::{FaucetClient, FaucetClientFactory},
};
use sui_types::base_types::SuiAddress;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

/// Start a Sui validator and fullnode for easy testing.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // TODO: Support a configuration directory for persisted networks:
    // /// Config directory that will be used to store network configuration
    // #[clap(short, long, parse(from_os_str), value_hint = ValueHint::DirPath)]
    // config: Option<std::path::PathBuf>,
    /// Port to start the Gateway RPC server on
    #[clap(long, default_value = "5001")]
    gateway_rpc_port: u16,

    /// Port to start the Fullnode RPC server on
    #[clap(long, default_value = "9000")]
    fullnode_rpc_port: u16,

    /// Port to start the fullnode websocket RPC server on
    #[clap(long, default_value = "9001")]
    websocket_rpc_port: u16,

    /// Port to start the Sui faucet on
    #[clap(long, default_value = "9123")]
    faucet_port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let cluster = LocalNewCluster::start(&ClusterTestOpt {
        env: Env::NewLocal,
        gateway_address: Some(format!("127.0.0.1:{}", args.gateway_rpc_port)),
        fullnode_address: Some(format!("127.0.0.1:{}", args.fullnode_rpc_port)),
        websocket_address: Some(format!("127.0.0.1:{}", args.websocket_rpc_port)),
        faucet_address: None,
    })
    .await?;

    println!("Fullnode RPC URL: {}", cluster.fullnode_url());
    println!(
        "Fullnode Websocket URL: {}",
        cluster.websocket_url().unwrap()
    );
    println!("Gateway RPC URL: {}", cluster.rpc_url());

    start_faucet(&cluster, args.faucet_port).await?;

    Ok(())
}

struct AppState {
    faucet: Arc<dyn FaucetClient + Sync + Send>,
}

async fn start_faucet(cluster: &LocalNewCluster, port: u16) -> Result<()> {
    let faucet = FaucetClientFactory::new_from_cluster(cluster).await;

    let app_state = Arc::new(AppState { faucet });

    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(health))
        .route("/faucet", post(faucet_request))
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(Extension(app_state))
                .into_inner(),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    println!("Faucet URL: http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// basic handler that responds with a static string
async fn health() -> &'static str {
    "OK"
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FaucetRequest {
    pub recipient: SuiAddress,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FaucetResponse {
    pub ok: bool,
}

async fn faucet_request(
    Json(payload): Json<FaucetRequest>,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    let result = state.faucet.request_sui_coins(payload.recipient).await;
    if !result.transferred_gas_objects.is_empty() {
        (StatusCode::OK, Json(FaucetResponse { ok: true }))
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FaucetResponse { ok: false }),
        )
    }
}
