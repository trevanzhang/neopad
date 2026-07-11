use axum::{
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use subtle::ConstantTimeEq;

#[derive(Debug, Clone, Copy)]
pub struct AuthError {
    status: StatusCode,
    message: &'static str,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}

#[derive(Debug, Clone)]
pub struct Auth {
    token: String,
}

impl Auth {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    pub fn validate(&self, headers: &HeaderMap) -> Result<(), AuthError> {
        self.validate_origin(headers)?;
        self.validate_bearer(headers)
    }

    fn validate_bearer(&self, headers: &HeaderMap) -> Result<(), AuthError> {
        let Some(value) = headers.get(header::AUTHORIZATION) else {
            return Err(plain(StatusCode::UNAUTHORIZED, "missing bearer token"));
        };
        let Ok(value) = value.to_str() else {
            return Err(plain(StatusCode::UNAUTHORIZED, "invalid bearer token"));
        };
        let Some(token) = value.strip_prefix("Bearer ") else {
            return Err(plain(StatusCode::UNAUTHORIZED, "invalid bearer token"));
        };
        if token.as_bytes().ct_eq(self.token.as_bytes()).unwrap_u8() != 1 {
            return Err(plain(StatusCode::UNAUTHORIZED, "invalid bearer token"));
        }
        Ok(())
    }

    fn validate_origin(&self, headers: &HeaderMap) -> Result<(), AuthError> {
        let Some(origin) = headers.get(header::ORIGIN) else {
            return Ok(());
        };
        let Ok(origin) = origin.to_str() else {
            return Err(plain(StatusCode::FORBIDDEN, "invalid origin"));
        };

        if is_local_origin(origin) {
            Ok(())
        } else {
            Err(plain(StatusCode::FORBIDDEN, "origin is not allowed"))
        }
    }
}

fn is_local_origin(origin: &str) -> bool {
    let Some(rest) = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
    else {
        return false;
    };
    let host_port = rest.split('/').next().unwrap_or(rest);
    let host = host_port
        .rsplit_once(':')
        .map_or(host_port, |(host, _)| host);
    matches!(host, "127.0.0.1" | "localhost" | "[::1]" | "::1")
}

fn plain(status: StatusCode, message: &'static str) -> AuthError {
    AuthError { status, message }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn local_origins_are_allowed() {
        assert!(is_local_origin("http://127.0.0.1:8765"));
        assert!(is_local_origin("http://localhost:3000"));
        assert!(is_local_origin("http://[::1]:8765"));
    }

    #[test]
    fn non_local_origins_are_rejected() {
        assert!(!is_local_origin("https://example.com"));
        assert!(!is_local_origin("file://local"));
    }

    #[test]
    fn missing_origin_is_allowed_when_token_is_valid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer secret"),
        );
        assert!(Auth::new("secret".to_owned()).validate(&headers).is_ok());
    }
}
