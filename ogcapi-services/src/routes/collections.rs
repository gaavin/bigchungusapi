use axum::{
    extract::{Extension, Path},
    headers::HeaderMap,
    http::{header::LOCATION, StatusCode},
    Json,
    {routing::get, Router},
};
use chrono::Utc;
// use serde::Deserialize;
// use serde_with::{serde_as, DisplayFromStr};
use url::Position;

use ogcapi_types::common::{
    link_rel::{DATA, ITEMS, SELF},
    media_type::{GEO_JSON, JSON},
    Collection, Collections, Crs, Link,
};

use crate::{extractors::RemoteUrl, Result, State};

const CONFORMANCE: [&str; 3] = [
    "http://www.opengis.net/spec/ogcapi-common-1/1.0/req/core",
    "http://www.opengis.net/spec/ogcapi-common-2/1.0/req/collections",
    "http://www.opengis.net/spec/ogcapi_common-2/1.0/req/json",
];

async fn collections(
    // Query(query): Query<CollectionQuery>,
    RemoteUrl(url): RemoteUrl,
    Extension(state): Extension<State>,
) -> Result<Json<Collections>> {
    let mut collections =
        sqlx::query_scalar!(
            r#"
            SELECT array_to_json(array_agg(collection)) as "collections!: sqlx::types::Json<Vec<Collection>>" 
            FROM meta.collections
            "#)
            .fetch_one(&state.db.pool)
            .await?;

    let base = &url[..Position::AfterPath];
    collections.0.iter_mut().for_each(|c| {
        c.links.append(&mut vec![
            Link::new(format!("{}/{}", base, c.id), SELF),
            Link::new(format!("{}/{}/items", base, c.id), ITEMS)
                .mime(GEO_JSON)
                .title(format!("Items of {}", c.title.as_ref().unwrap_or(&c.id))),
        ]);
    });

    let collections = Collections {
        links: vec![Link::new(url, SELF).mime(JSON).title("this document")],
        time_stamp: Some(Utc::now().to_rfc3339()),
        crs: vec![Crs::default(), Crs::from(4326)],
        collections: collections.0,
        ..Default::default()
    };

    Ok(Json(collections))
}

/// Create new collection metadata
async fn insert(
    Json(collection): Json<Collection>,
    Extension(state): Extension<State>,
) -> Result<(StatusCode, HeaderMap)> {
    let location = state.db.insert_collection(&collection).await?;
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, location.parse().unwrap());
    Ok((StatusCode::CREATED, headers))
}

/// Get collection metadata
async fn read(
    Path(collection_id): Path<String>,
    RemoteUrl(url): RemoteUrl,
    Extension(state): Extension<State>,
) -> Result<Json<Collection>> {
    let mut collection = state.db.select_collection(&collection_id).await?;

    collection.links.push(
        Link::new(format!("{}/items", &url[..Position::AfterPath]), SELF)
            .mime(GEO_JSON)
            .title(format!(
                "Items of {}",
                collection.title.as_ref().unwrap_or(&collection.id)
            )),
    );

    Ok(Json(collection))
}

/// Update collection metadata
async fn update(
    Path(collection_id): Path<String>,
    Json(mut collection): Json<Collection>,
    Extension(state): Extension<State>,
) -> Result<StatusCode> {
    collection.id = collection_id;

    state.db.update_collection(&collection).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Delete collection metadata
async fn remove(
    Path(collection_id): Path<String>,
    Extension(state): Extension<State>,
) -> Result<StatusCode> {
    state.db.delete_collection(&collection_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub(crate) fn router(state: &State) -> Router {
    let mut root = state.root.write().unwrap();
    root.links.push(
        Link::new(format!("{}/collections", &state.remote), DATA)
            .title("Metadata about the resource collections")
            .mime(JSON),
    );

    let mut conformance = state.conformance.write().unwrap();
    conformance
        .conforms_to
        .append(&mut CONFORMANCE.map(String::from).to_vec());

    Router::new()
        .route("/collections", get(collections).post(insert))
        .route(
            "/collections/:collection_id",
            get(read).put(update).delete(remove),
        )
}
