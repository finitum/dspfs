use crate::dspfs::api::Api;
use crate::dspfs::Dspfs;
use crate::global_store::Store;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use warp::{http, reject, Filter};

#[derive(Debug)]
struct CustomReject(anyhow::Error);

impl warp::reject::Reject for CustomReject {}

fn json_body<T: Send + Sync + for<'de> Deserialize<'de>>(
) -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

struct RestApi<'dspfs, S> {
    dspfs: Arc<&'dspfs Dspfs<S>>,
}

impl<'dspfs, S: Store> RestApi<'dspfs, S> {
    pub fn new(dspfs: Arc<&'dspfs Dspfs<S>>) -> Self {
        Self { dspfs }
    }

    async fn init_group(
        dspfs: Arc<&Dspfs<S>>,
        path: PathBuf,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        dspfs
            .init_group(&path)
            .await
            .map_err(|e| reject::custom(CustomReject(e)))?;

        Ok(warp::reply::with_status(
            "Initialized group",
            http::StatusCode::CREATED,
        ))
    }

    pub async fn serve(self) {
        let dspfs = self.dspfs.clone();

        let dspfs_filter = warp::any().map(move || dspfs.clone());

        let initgroup = warp::post()
            .and(warp::path("group"))
            .and(warp::path("init"))
            .and(warp::path::end())
            .and(dspfs_filter)
            .and(json_body::<PathBuf>())
            .and_then(Self::init_group);

        let routes = initgroup.with(warp::cors().allow_any_origin());
    }
}
