use crate::models::crates::get_crate_with_permissions;
use axum::{body::Full, extract, response::IntoResponse, Json};
use bytes::Bytes;
use chartered_db::{
    crates::Crate,
    users::{User, UserCratePermissionValue as Permission},
    ConnectionPool,
};
use chartered_types::cargo::CrateVersion;
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to query database")]
    Database(#[from] chartered_db::Error),
    #[error("Failed to fetch crate file")]
    File(#[from] std::io::Error),
    #[error("{0}")]
    CrateFetch(#[from] crate::models::crates::CrateFetchError),
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(_) | Self::File(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::CrateFetch(e) => e.status_code(),
        }
    }
}

define_error_response!(Error);

pub async fn handle(
    extract::Path((_session_key, name)): extract::Path<(String, String)>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
) -> Result<axum::http::Response<Full<Bytes>>, Error> {
    let crate_ = get_crate_with_permissions(db.clone(), user, name, &[Permission::VISIBLE]).await?;
    let versions = crate_.clone().versions(db).await?;

    // returning a Response instead of Json here so we don't have to close
    // every Crate/CrateVersion etc, would be easier if we just had an owned
    // version of each but we're using `spawn_blocking` in chartered-db for
    // diesel which requires `'static' which basically forces us to use Arc
    // if we want to keep a reference to anything ourselves.
    Ok(Json(Response {
        info: crate_.as_ref().into(),
        versions: versions
            .into_iter()
            .map(|v| v.into_cargo_format(&crate_))
            .collect(),
    })
    .into_response())
}

#[derive(Serialize)]
pub struct Response<'a> {
    #[serde(flatten)]
    info: ResponseInfo<'a>,
    versions: Vec<CrateVersion<'a>>,
}

#[derive(Serialize)]
pub struct ResponseInfo<'a> {
    name: &'a str,
    readme: Option<&'a str>,
    description: Option<&'a str>,
    repository: Option<&'a str>,
    homepage: Option<&'a str>,
    documentation: Option<&'a str>,
}

impl<'a> From<&'a Crate> for ResponseInfo<'a> {
    fn from(crate_: &'a Crate) -> Self {
        Self {
            name: &crate_.name,
            readme: crate_.readme.as_deref(),
            description: crate_.description.as_deref(),
            repository: crate_.repository.as_deref(),
            homepage: crate_.homepage.as_deref(),
            documentation: crate_.documentation.as_deref(),
        }
    }
}
