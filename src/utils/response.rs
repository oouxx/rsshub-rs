use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

pub struct RssResponse(pub String);

impl IntoResponse for RssResponse {
    fn into_response(self) -> Response {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
            self.0,
        )
            .into_response()
    }
}

pub struct ErrorResponse(pub String);

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}
