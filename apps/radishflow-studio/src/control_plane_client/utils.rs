use rf_types::{RfError, RfResult};

use super::{
    RadishFlowControlPlaneClientError, RadishFlowControlPlaneHttpTransportError,
    RadishFlowControlPlaneHttpTransportErrorKind,
};

pub(super) fn validate_access_token(
    access_token: &str,
) -> Result<(), RadishFlowControlPlaneClientError> {
    if access_token.trim().is_empty() {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            "control plane access token must not be empty",
        ));
    }

    Ok(())
}

pub(super) fn validate_package_id(
    package_id: &str,
) -> Result<(), RadishFlowControlPlaneClientError> {
    if package_id.trim().is_empty() {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            "property package id must not be empty",
        ));
    }

    Ok(())
}

pub(super) fn normalize_base_url(base_url: String) -> RfResult<String> {
    let normalized = base_url.trim().trim_end_matches('/').to_string();
    if normalized.is_empty() {
        return Err(RfError::invalid_input(
            "radishflow control plane base_url must not be empty",
        ));
    }

    Ok(normalized)
}

pub(super) fn percent_encode_path_segment(segment: &str) -> String {
    let mut encoded = String::new();
    for byte in segment.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push_str(&format!("{byte:02X}"));
        }
    }
    encoded
}

pub(super) fn map_http_transport_error(
    error: RadishFlowControlPlaneHttpTransportError,
) -> RadishFlowControlPlaneClientError {
    match error.kind {
        RadishFlowControlPlaneHttpTransportErrorKind::Timeout => {
            RadishFlowControlPlaneClientError::timeout(error.message)
        }
        RadishFlowControlPlaneHttpTransportErrorKind::ConnectionUnavailable => {
            RadishFlowControlPlaneClientError::connection_unavailable(error.message)
        }
        RadishFlowControlPlaneHttpTransportErrorKind::OtherTransient => {
            RadishFlowControlPlaneClientError::other_transient(error.message)
        }
        RadishFlowControlPlaneHttpTransportErrorKind::OtherPermanent => {
            RadishFlowControlPlaneClientError::other_permanent(error.message)
        }
    }
}

pub(super) fn map_reqwest_transport_error(
    error: reqwest::Error,
) -> RadishFlowControlPlaneHttpTransportError {
    if error.is_connect() {
        RadishFlowControlPlaneHttpTransportError::connection_unavailable(error.to_string())
    } else if error.is_timeout() {
        RadishFlowControlPlaneHttpTransportError::timeout(error.to_string())
    } else if error.is_body() {
        RadishFlowControlPlaneHttpTransportError::other_transient(error.to_string())
    } else {
        RadishFlowControlPlaneHttpTransportError::other_permanent(error.to_string())
    }
}
