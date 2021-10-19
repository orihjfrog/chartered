use axum::{
    body::{Full, HttpBody},
    extract,
    http::Response,
    response::{IntoResponse, Redirect},
};
use bytes::Bytes;
use chartered_db::{crates::Crate, users::User, ConnectionPool};
use chartered_fs::{FilePointer, FileSystem};
use std::{str::FromStr, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] chartered_db::Error),
    #[error("Failed to fetch crate file: {0}")]
    File(#[from] Box<chartered_fs::Error>),
    #[error("The requested version does not exist for the crate")]
    NoVersion,
}

impl Error {
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::Database(e) => e.status_code(),
            Self::File(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NoVersion => StatusCode::NOT_FOUND,
        }
    }
}

define_error_response!(Error);

pub enum ResponseOrRedirect {
    Response(Vec<u8>),
    Redirect(Redirect),
}

impl IntoResponse for ResponseOrRedirect {
    type Body = Full<Bytes>;
    type BodyError = <Self::Body as HttpBody>::Error;

    fn into_response(self) -> Response<Self::Body> {
        match self {
            Self::Response(v) => v.into_response(),
            Self::Redirect(v) => v.into_response().map(|_| Full::from(Bytes::new())),
        }
    }
}

pub async fn handle(
    extract::Path((_session_key, organisation, name, version)): extract::Path<(
        String,
        String,
        String,
        String,
    )>,
    extract::Extension(db): extract::Extension<ConnectionPool>,
    extract::Extension(user): extract::Extension<Arc<User>>,
    extract::Extension(fs): extract::Extension<Arc<FileSystem>>,
) -> Result<ResponseOrRedirect, Error> {
    let crate_with_permissions =
        Arc::new(Crate::find_by_name(db.clone(), user.id, organisation, name).await?);

    // we shouldn't really hold back this request from progressing whilst waiting
    // on the downloads increment to complete so we'll just tokio::spawn it
    tokio::spawn(
        crate_with_permissions
            .clone()
            .increment_download_count(db.clone()),
    );

    let version = crate_with_permissions
        .version(db, version)
        .await?
        .ok_or(Error::NoVersion)?;

    let file_ref = chartered_fs::FileReference::from_str(&version.filesystem_object).unwrap();

    let res = fs.read(file_ref).await.map_err(Box::new)?;

    match res {
        FilePointer::Redirect(uri) => Ok(ResponseOrRedirect::Redirect(Redirect::to(uri))),
        FilePointer::Content(content) => Ok(ResponseOrRedirect::Response(content)),
    }
}
