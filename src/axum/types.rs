use axum::{
    Form, Json,
    extract::{
        FromRequest, Request,
        rejection::{FormRejection, JsonRejection},
    },
    http::{StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};

pub(crate) enum FormOrJsonRejection {
    MissingContentType,
    UnsupportedContentType,
    Json(JsonRejection),
    Form(FormRejection),
}

impl IntoResponse for FormOrJsonRejection {
    fn into_response(self) -> Response {
        match self {
            Self::MissingContentType | Self::UnsupportedContentType => {
                StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response()
            }
            _ => StatusCode::BAD_REQUEST.into_response(),
        }
    }
}

pub(crate) enum FormOrJson<T> {
    Form(T),
    Json(T),
}

impl<T> FormOrJson<T> {
    pub(crate) fn into_inner(self) -> T {
        match self {
            Self::Form(value) | Self::Json(value) => value,
        }
    }
}

impl<S, T> FromRequest<S> for FormOrJson<T>
where
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
    S: Send + Sync,
{
    type Rejection = FormOrJsonRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.split(';').next().unwrap_or("").trim())
            .ok_or(FormOrJsonRejection::MissingContentType)?;

        match content_type {
            "application/json" => {
                let Json(payload) = Json::<T>::from_request(req, state)
                    .await
                    .map_err(FormOrJsonRejection::Json)?;

                Ok(FormOrJson::Json(payload))
            }
            "application/x-www-form-urlencoded" => {
                let Form(payload) = Form::from_request(req, state)
                    .await
                    .map_err(FormOrJsonRejection::Form)?;

                Ok(FormOrJson::Form(payload))
            }
            _ => Err(FormOrJsonRejection::UnsupportedContentType),
        }
    }
}
