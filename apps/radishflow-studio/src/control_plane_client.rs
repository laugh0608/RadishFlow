use std::time::{Duration, SystemTime};

use self::utils::{
    map_http_transport_error, map_reqwest_transport_error, normalize_base_url,
    percent_encode_path_segment, validate_access_token, validate_package_id,
};
use rf_store::{option_time_format, time_format};
use rf_types::{RfError, RfResult};
use rf_ui::{
    EntitlementSnapshot, OfflineLeaseRefreshRequest, OfflineLeaseRefreshResponse,
    PropertyPackageClassification, PropertyPackageLeaseGrant, PropertyPackageLeaseRequest,
    PropertyPackageManifest, PropertyPackageManifestList, PropertyPackageSource,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub const RADISHFLOW_CONTROL_PLANE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadishFlowControlPlaneEndpoints {
    base_url: String,
}

impl RadishFlowControlPlaneEndpoints {
    pub fn new(base_url: impl Into<String>) -> RfResult<Self> {
        let normalized = normalize_base_url(base_url.into())?;
        Ok(Self {
            base_url: normalized,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn entitlement_snapshot_url(&self) -> String {
        self.join_path("/api/radishflow/entitlements/current")
    }

    pub fn property_package_manifest_url(&self) -> String {
        self.join_path("/api/radishflow/property-packages/manifest")
    }

    pub fn property_package_lease_url(&self, package_id: &str) -> String {
        self.join_path(&format!(
            "/api/radishflow/property-packages/{}/lease",
            percent_encode_path_segment(package_id)
        ))
    }

    pub fn offline_lease_refresh_url(&self) -> String {
        self.join_path("/api/radishflow/offline-leases/refresh")
    }

    fn join_path(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RadishFlowControlPlaneHttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadishFlowControlPlaneHttpRequest {
    pub method: RadishFlowControlPlaneHttpMethod,
    pub url: String,
    pub bearer_token: String,
    pub accept_content_types: Vec<String>,
    pub content_type: Option<String>,
    pub body: Option<String>,
}

impl RadishFlowControlPlaneHttpRequest {
    pub fn new_get(url: impl Into<String>, bearer_token: impl Into<String>) -> Self {
        Self {
            method: RadishFlowControlPlaneHttpMethod::Get,
            url: url.into(),
            bearer_token: bearer_token.into(),
            accept_content_types: vec!["application/json".to_string()],
            content_type: None,
            body: None,
        }
    }

    pub fn new_post_json(
        url: impl Into<String>,
        bearer_token: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            method: RadishFlowControlPlaneHttpMethod::Post,
            url: url.into(),
            bearer_token: bearer_token.into(),
            accept_content_types: vec!["application/json".to_string()],
            content_type: Some("application/json".to_string()),
            body: Some(body.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadishFlowControlPlaneHttpResponse {
    pub status_code: u16,
    pub body: String,
    pub content_type: Option<String>,
    pub received_at: SystemTime,
}

impl RadishFlowControlPlaneHttpResponse {
    pub fn new(status_code: u16, body: impl Into<String>, received_at: SystemTime) -> Self {
        Self {
            status_code,
            body: body.into(),
            content_type: Some("application/json".to_string()),
            received_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RadishFlowControlPlaneHttpTransportErrorKind {
    Timeout,
    ConnectionUnavailable,
    OtherTransient,
    OtherPermanent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadishFlowControlPlaneHttpTransportError {
    pub kind: RadishFlowControlPlaneHttpTransportErrorKind,
    pub message: String,
}

impl RadishFlowControlPlaneHttpTransportError {
    pub fn new(
        kind: RadishFlowControlPlaneHttpTransportErrorKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(
            RadishFlowControlPlaneHttpTransportErrorKind::Timeout,
            message,
        )
    }

    pub fn connection_unavailable(message: impl Into<String>) -> Self {
        Self::new(
            RadishFlowControlPlaneHttpTransportErrorKind::ConnectionUnavailable,
            message,
        )
    }

    pub fn other_transient(message: impl Into<String>) -> Self {
        Self::new(
            RadishFlowControlPlaneHttpTransportErrorKind::OtherTransient,
            message,
        )
    }

    pub fn other_permanent(message: impl Into<String>) -> Self {
        Self::new(
            RadishFlowControlPlaneHttpTransportErrorKind::OtherPermanent,
            message,
        )
    }
}

pub trait RadishFlowControlPlaneHttpTransport {
    fn send(
        &self,
        request: &RadishFlowControlPlaneHttpRequest,
    ) -> Result<RadishFlowControlPlaneHttpResponse, RadishFlowControlPlaneHttpTransportError>;
}

impl<Transport> RadishFlowControlPlaneHttpTransport for &Transport
where
    Transport: RadishFlowControlPlaneHttpTransport,
{
    fn send(
        &self,
        request: &RadishFlowControlPlaneHttpRequest,
    ) -> Result<RadishFlowControlPlaneHttpResponse, RadishFlowControlPlaneHttpTransportError> {
        (*self).send(request)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RadishFlowControlPlaneClientErrorKind {
    Timeout,
    ConnectionUnavailable,
    RateLimited,
    ServiceUnavailable,
    Unauthorized,
    Forbidden,
    NotFound,
    InvalidResponse,
    OtherTransient,
    OtherPermanent,
}

impl RadishFlowControlPlaneClientErrorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::ConnectionUnavailable => "connection-unavailable",
            Self::RateLimited => "rate-limited",
            Self::ServiceUnavailable => "service-unavailable",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not-found",
            Self::InvalidResponse => "invalid-response",
            Self::OtherTransient => "other-transient",
            Self::OtherPermanent => "other-permanent",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadishFlowControlPlaneClientError {
    pub kind: RadishFlowControlPlaneClientErrorKind,
    pub message: String,
}

impl RadishFlowControlPlaneClientError {
    pub fn new(kind: RadishFlowControlPlaneClientErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(RadishFlowControlPlaneClientErrorKind::Timeout, message)
    }

    pub fn connection_unavailable(message: impl Into<String>) -> Self {
        Self::new(
            RadishFlowControlPlaneClientErrorKind::ConnectionUnavailable,
            message,
        )
    }

    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self::new(RadishFlowControlPlaneClientErrorKind::RateLimited, message)
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(
            RadishFlowControlPlaneClientErrorKind::ServiceUnavailable,
            message,
        )
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(RadishFlowControlPlaneClientErrorKind::Unauthorized, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(RadishFlowControlPlaneClientErrorKind::Forbidden, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(RadishFlowControlPlaneClientErrorKind::NotFound, message)
    }

    pub fn invalid_response(message: impl Into<String>) -> Self {
        Self::new(
            RadishFlowControlPlaneClientErrorKind::InvalidResponse,
            message,
        )
    }

    pub fn other_transient(message: impl Into<String>) -> Self {
        Self::new(
            RadishFlowControlPlaneClientErrorKind::OtherTransient,
            message,
        )
    }

    pub fn other_permanent(message: impl Into<String>) -> Self {
        Self::new(
            RadishFlowControlPlaneClientErrorKind::OtherPermanent,
            message,
        )
    }

    pub fn into_rf_error(self, operation: &str) -> RfError {
        RfError::invalid_input(format!(
            "{operation} failed with {} error: {}",
            self.kind.as_str(),
            self.message
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadishFlowControlPlaneResponse<T> {
    pub value: T,
    pub received_at: SystemTime,
}

impl<T> RadishFlowControlPlaneResponse<T> {
    pub fn new(value: T, received_at: SystemTime) -> Self {
        Self { value, received_at }
    }
}

pub trait RadishFlowControlPlaneClient {
    fn fetch_entitlement_snapshot(
        &self,
        access_token: &str,
    ) -> Result<
        RadishFlowControlPlaneResponse<EntitlementSnapshot>,
        RadishFlowControlPlaneClientError,
    >;

    fn fetch_property_package_manifest_list(
        &self,
        access_token: &str,
    ) -> Result<
        RadishFlowControlPlaneResponse<PropertyPackageManifestList>,
        RadishFlowControlPlaneClientError,
    >;

    fn request_property_package_lease(
        &self,
        access_token: &str,
        package_id: &str,
        request: &PropertyPackageLeaseRequest,
    ) -> Result<
        RadishFlowControlPlaneResponse<PropertyPackageLeaseGrant>,
        RadishFlowControlPlaneClientError,
    >;

    fn refresh_offline_leases(
        &self,
        access_token: &str,
        request: &OfflineLeaseRefreshRequest,
    ) -> Result<
        RadishFlowControlPlaneResponse<OfflineLeaseRefreshResponse>,
        RadishFlowControlPlaneClientError,
    >;
}

#[derive(Debug, Clone)]
pub struct HttpRadishFlowControlPlaneClient<Transport> {
    endpoints: RadishFlowControlPlaneEndpoints,
    transport: Transport,
}

impl<Transport> HttpRadishFlowControlPlaneClient<Transport> {
    pub fn new(endpoints: RadishFlowControlPlaneEndpoints, transport: Transport) -> Self {
        Self {
            endpoints,
            transport,
        }
    }

    pub fn endpoints(&self) -> &RadishFlowControlPlaneEndpoints {
        &self.endpoints
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReqwestRadishFlowControlPlaneHttpTransportOptions {
    pub request_timeout: Duration,
    pub user_agent: String,
}

impl Default for ReqwestRadishFlowControlPlaneHttpTransportOptions {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            user_agent: format!("radishflow-studio/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReqwestRadishFlowControlPlaneHttpTransport {
    client: reqwest::blocking::Client,
}

impl ReqwestRadishFlowControlPlaneHttpTransport {
    pub fn new() -> RfResult<Self> {
        Self::with_options(ReqwestRadishFlowControlPlaneHttpTransportOptions::default())
    }

    pub fn with_options(
        options: ReqwestRadishFlowControlPlaneHttpTransportOptions,
    ) -> RfResult<Self> {
        if options.request_timeout == Duration::ZERO {
            return Err(RfError::invalid_input(
                "reqwest control plane transport timeout must be greater than zero",
            ));
        }

        if options.user_agent.trim().is_empty() {
            return Err(RfError::invalid_input(
                "reqwest control plane transport user_agent must not be empty",
            ));
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(options.request_timeout)
            .user_agent(options.user_agent)
            .build()
            .map_err(|error| {
                RfError::invalid_input(format!("build reqwest control plane transport: {error}"))
            })?;

        Ok(Self { client })
    }
}

impl<Transport> RadishFlowControlPlaneClient for HttpRadishFlowControlPlaneClient<Transport>
where
    Transport: RadishFlowControlPlaneHttpTransport,
{
    fn fetch_entitlement_snapshot(
        &self,
        access_token: &str,
    ) -> Result<
        RadishFlowControlPlaneResponse<EntitlementSnapshot>,
        RadishFlowControlPlaneClientError,
    > {
        validate_access_token(access_token)?;
        let request = RadishFlowControlPlaneHttpRequest::new_get(
            self.endpoints.entitlement_snapshot_url(),
            access_token.to_string(),
        );
        let response: RadishFlowControlPlaneResponse<ProtocolEntitlementSnapshot> =
            self.send_json_request(request, "fetch entitlement snapshot")?;
        Ok(RadishFlowControlPlaneResponse::new(
            entitlement_snapshot_from_protocol(response.value)?,
            response.received_at,
        ))
    }

    fn fetch_property_package_manifest_list(
        &self,
        access_token: &str,
    ) -> Result<
        RadishFlowControlPlaneResponse<PropertyPackageManifestList>,
        RadishFlowControlPlaneClientError,
    > {
        validate_access_token(access_token)?;
        let request = RadishFlowControlPlaneHttpRequest::new_get(
            self.endpoints.property_package_manifest_url(),
            access_token.to_string(),
        );
        let response: RadishFlowControlPlaneResponse<ProtocolPropertyPackageManifestList> =
            self.send_json_request(request, "fetch property package manifest list")?;
        Ok(RadishFlowControlPlaneResponse::new(
            property_package_manifest_list_from_protocol(response.value)?,
            response.received_at,
        ))
    }

    fn request_property_package_lease(
        &self,
        access_token: &str,
        package_id: &str,
        request: &PropertyPackageLeaseRequest,
    ) -> Result<
        RadishFlowControlPlaneResponse<PropertyPackageLeaseGrant>,
        RadishFlowControlPlaneClientError,
    > {
        validate_access_token(access_token)?;
        validate_package_id(package_id)?;
        let body =
            serde_json::to_string(&ProtocolPropertyPackageLeaseRequest::from_runtime(request)?)
                .map_err(|error| {
                    RadishFlowControlPlaneClientError::invalid_response(format!(
                        "serialize property package lease request: {error}"
                    ))
                })?;
        let request = RadishFlowControlPlaneHttpRequest::new_post_json(
            self.endpoints.property_package_lease_url(package_id),
            access_token.to_string(),
            body,
        );
        let response: RadishFlowControlPlaneResponse<ProtocolPropertyPackageLeaseGrant> =
            self.send_json_request(request, "request property package lease")?;
        Ok(RadishFlowControlPlaneResponse::new(
            property_package_lease_grant_from_protocol(response.value)?,
            response.received_at,
        ))
    }

    fn refresh_offline_leases(
        &self,
        access_token: &str,
        request: &OfflineLeaseRefreshRequest,
    ) -> Result<
        RadishFlowControlPlaneResponse<OfflineLeaseRefreshResponse>,
        RadishFlowControlPlaneClientError,
    > {
        validate_access_token(access_token)?;
        let body =
            serde_json::to_string(&ProtocolOfflineLeaseRefreshRequest::from_runtime(request))
                .map_err(|error| {
                    RadishFlowControlPlaneClientError::invalid_response(format!(
                        "serialize offline lease refresh request: {error}"
                    ))
                })?;
        let request = RadishFlowControlPlaneHttpRequest::new_post_json(
            self.endpoints.offline_lease_refresh_url(),
            access_token.to_string(),
            body,
        );
        let response: RadishFlowControlPlaneResponse<ProtocolOfflineLeaseRefreshResponse> =
            self.send_json_request(request, "refresh offline lease")?;
        Ok(RadishFlowControlPlaneResponse::new(
            offline_lease_refresh_response_from_protocol(response.value)?,
            response.received_at,
        ))
    }
}

impl<Transport> HttpRadishFlowControlPlaneClient<Transport>
where
    Transport: RadishFlowControlPlaneHttpTransport,
{
    fn send_json_request<Response>(
        &self,
        request: RadishFlowControlPlaneHttpRequest,
        operation: &str,
    ) -> Result<RadishFlowControlPlaneResponse<Response>, RadishFlowControlPlaneClientError>
    where
        Response: DeserializeOwned,
    {
        let response = self
            .transport
            .send(&request)
            .map_err(map_http_transport_error)?;
        parse_json_response(response, operation)
    }
}

impl RadishFlowControlPlaneHttpTransport for ReqwestRadishFlowControlPlaneHttpTransport {
    fn send(
        &self,
        request: &RadishFlowControlPlaneHttpRequest,
    ) -> Result<RadishFlowControlPlaneHttpResponse, RadishFlowControlPlaneHttpTransportError> {
        let accept_header = request.accept_content_types.join(", ");
        let mut builder = match request.method {
            RadishFlowControlPlaneHttpMethod::Get => self.client.get(&request.url),
            RadishFlowControlPlaneHttpMethod::Post => self.client.post(&request.url),
        };

        builder = builder
            .header(reqwest::header::ACCEPT, accept_header)
            .bearer_auth(&request.bearer_token);

        if let Some(content_type) = &request.content_type {
            builder = builder.header(reqwest::header::CONTENT_TYPE, content_type);
        }

        if let Some(body) = &request.body {
            builder = builder.body(body.clone());
        }

        let response = builder.send().map_err(map_reqwest_transport_error)?;
        let status_code = response.status().as_u16();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let received_at = SystemTime::now();
        let body = response.text().map_err(map_reqwest_transport_error)?;

        Ok(RadishFlowControlPlaneHttpResponse {
            status_code,
            body,
            content_type,
            received_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProtocolEntitlementSnapshot {
    schema_version: u32,
    subject_id: String,
    tenant_id: Option<String>,
    #[serde(with = "time_format")]
    issued_at: SystemTime,
    #[serde(with = "time_format")]
    expires_at: SystemTime,
    #[serde(default, with = "option_time_format")]
    offline_lease_expires_at: Option<SystemTime>,
    #[serde(default)]
    features: std::collections::BTreeSet<String>,
    #[serde(default)]
    allowed_package_ids: std::collections::BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProtocolPropertyPackageManifest {
    schema_version: u32,
    package_id: String,
    version: String,
    classification: ProtocolPropertyPackageClassification,
    source: ProtocolPropertyPackageSource,
    hash: String,
    size_bytes: u64,
    #[serde(default)]
    component_ids: Vec<rf_types::ComponentId>,
    lease_required: bool,
    #[serde(default, with = "option_time_format")]
    expires_at: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProtocolPropertyPackageManifestList {
    schema_version: u32,
    #[serde(with = "time_format")]
    generated_at: SystemTime,
    #[serde(default)]
    packages: Vec<ProtocolPropertyPackageManifest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
enum ProtocolPropertyPackageClassification {
    #[serde(rename = "derived")]
    Derived,
    #[serde(rename = "remote-only")]
    RemoteOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
enum ProtocolPropertyPackageSource {
    #[serde(rename = "bundled")]
    Bundled,
    #[serde(rename = "download")]
    Download,
    #[serde(rename = "remote-eval")]
    RemoteEval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProtocolPropertyPackageLeaseRequest {
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    installation_id: Option<String>,
}

impl ProtocolPropertyPackageLeaseRequest {
    fn from_runtime(
        request: &PropertyPackageLeaseRequest,
    ) -> Result<Self, RadishFlowControlPlaneClientError> {
        if request.version.trim().is_empty() {
            return Err(RadishFlowControlPlaneClientError::invalid_response(
                "property package lease request must contain a non-empty version",
            ));
        }

        Ok(Self {
            version: request.version.clone(),
            current_hash: request.current_hash.clone(),
            installation_id: request.installation_id.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProtocolPropertyPackageLeaseGrant {
    package_id: String,
    version: String,
    lease_id: String,
    download_url: String,
    hash: String,
    size_bytes: u64,
    #[serde(with = "time_format")]
    expires_at: SystemTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProtocolOfflineLeaseRefreshRequest {
    package_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", with = "option_time_format")]
    current_offline_lease_expires_at: Option<SystemTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    installation_id: Option<String>,
}

impl ProtocolOfflineLeaseRefreshRequest {
    fn from_runtime(request: &OfflineLeaseRefreshRequest) -> Self {
        Self {
            package_ids: request.package_ids.iter().cloned().collect(),
            current_offline_lease_expires_at: request.current_offline_lease_expires_at,
            installation_id: request.installation_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProtocolOfflineLeaseRefreshResponse {
    #[serde(with = "time_format")]
    refreshed_at: SystemTime,
    snapshot: ProtocolEntitlementSnapshot,
    manifest_list: ProtocolPropertyPackageManifestList,
}

fn parse_json_response<Response>(
    response: RadishFlowControlPlaneHttpResponse,
    operation: &str,
) -> Result<RadishFlowControlPlaneResponse<Response>, RadishFlowControlPlaneClientError>
where
    Response: DeserializeOwned,
{
    match response.status_code {
        200..=299 => {
            if let Some(content_type) = &response.content_type
                && !content_type
                    .to_ascii_lowercase()
                    .contains("application/json")
            {
                return Err(RadishFlowControlPlaneClientError::invalid_response(
                    format!("{operation} returned unsupported content-type `{content_type}`"),
                ));
            }

            if response.body.trim().is_empty() {
                return Err(RadishFlowControlPlaneClientError::invalid_response(
                    format!("{operation} returned an empty successful body"),
                ));
            }

            let value = serde_json::from_str(&response.body).map_err(|error| {
                RadishFlowControlPlaneClientError::invalid_response(format!(
                    "{operation} returned invalid JSON: {error}"
                ))
            })?;
            Ok(RadishFlowControlPlaneResponse::new(
                value,
                response.received_at,
            ))
        }
        401 => Err(RadishFlowControlPlaneClientError::unauthorized(format!(
            "{operation} is no longer authorized"
        ))),
        403 => Err(RadishFlowControlPlaneClientError::forbidden(format!(
            "{operation} was rejected by the control plane"
        ))),
        404 => Err(RadishFlowControlPlaneClientError::not_found(format!(
            "{operation} endpoint was not found"
        ))),
        408 | 504 => Err(RadishFlowControlPlaneClientError::timeout(format!(
            "{operation} timed out with HTTP {}",
            response.status_code
        ))),
        429 => Err(RadishFlowControlPlaneClientError::rate_limited(format!(
            "{operation} was rate limited"
        ))),
        500..=599 => Err(RadishFlowControlPlaneClientError::service_unavailable(
            format!(
                "{operation} failed with server status {}",
                response.status_code
            ),
        )),
        _ => Err(RadishFlowControlPlaneClientError::invalid_response(
            format!(
                "{operation} returned unsupported HTTP status {}",
                response.status_code
            ),
        )),
    }
}

fn entitlement_snapshot_from_protocol(
    snapshot: ProtocolEntitlementSnapshot,
) -> Result<EntitlementSnapshot, RadishFlowControlPlaneClientError> {
    validate_protocol_schema_version(snapshot.schema_version, "entitlement snapshot")?;
    if snapshot.subject_id.trim().is_empty() {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            "entitlement snapshot must contain a non-empty subjectId",
        ));
    }

    Ok(EntitlementSnapshot {
        schema_version: snapshot.schema_version,
        subject_id: snapshot.subject_id,
        tenant_id: snapshot.tenant_id,
        issued_at: snapshot.issued_at,
        expires_at: snapshot.expires_at,
        offline_lease_expires_at: snapshot.offline_lease_expires_at,
        features: snapshot.features,
        allowed_package_ids: snapshot.allowed_package_ids,
    })
}

fn property_package_manifest_from_protocol(
    manifest: ProtocolPropertyPackageManifest,
) -> Result<PropertyPackageManifest, RadishFlowControlPlaneClientError> {
    validate_protocol_schema_version(manifest.schema_version, "property package manifest")?;
    if manifest.package_id.trim().is_empty() {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            "property package manifest must contain a non-empty packageId",
        ));
    }
    if manifest.version.trim().is_empty() {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            "property package manifest must contain a non-empty version",
        ));
    }

    let source = property_package_source_from_protocol(manifest.source);
    let expected_classification =
        property_package_classification_from_protocol(manifest.classification);
    let mut runtime = PropertyPackageManifest::new(manifest.package_id, manifest.version, source);
    if runtime.classification != expected_classification {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            format!(
                "property package manifest classification `{:?}` does not match source `{:?}`",
                expected_classification, source
            ),
        ));
    }
    if runtime.lease_required != manifest.lease_required {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            format!(
                "property package manifest leaseRequired `{}` does not match source `{:?}`",
                manifest.lease_required, source
            ),
        ));
    }

    runtime.schema_version = manifest.schema_version;
    runtime.hash = manifest.hash;
    runtime.size_bytes = manifest.size_bytes;
    runtime.component_ids = manifest.component_ids;
    runtime.expires_at = manifest.expires_at;
    Ok(runtime)
}

fn property_package_manifest_list_from_protocol(
    manifest_list: ProtocolPropertyPackageManifestList,
) -> Result<PropertyPackageManifestList, RadishFlowControlPlaneClientError> {
    validate_protocol_schema_version(
        manifest_list.schema_version,
        "property package manifest list",
    )?;
    let packages = manifest_list
        .packages
        .into_iter()
        .map(property_package_manifest_from_protocol)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PropertyPackageManifestList {
        schema_version: manifest_list.schema_version,
        generated_at: manifest_list.generated_at,
        packages,
    })
}

fn property_package_lease_grant_from_protocol(
    grant: ProtocolPropertyPackageLeaseGrant,
) -> Result<PropertyPackageLeaseGrant, RadishFlowControlPlaneClientError> {
    if grant.package_id.trim().is_empty() {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            "property package lease grant must contain a non-empty packageId",
        ));
    }
    if grant.version.trim().is_empty() {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            "property package lease grant must contain a non-empty version",
        ));
    }
    if grant.lease_id.trim().is_empty() {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            "property package lease grant must contain a non-empty leaseId",
        ));
    }
    if grant.download_url.trim().is_empty() {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            "property package lease grant must contain a non-empty downloadUrl",
        ));
    }
    if grant.hash.trim().is_empty() {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            "property package lease grant must contain a non-empty hash",
        ));
    }
    if grant.size_bytes == 0 {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            "property package lease grant must contain a non-zero sizeBytes",
        ));
    }

    Ok(PropertyPackageLeaseGrant {
        package_id: grant.package_id,
        version: grant.version,
        lease_id: grant.lease_id,
        download_url: grant.download_url,
        hash: grant.hash,
        size_bytes: grant.size_bytes,
        expires_at: grant.expires_at,
    })
}

fn offline_lease_refresh_response_from_protocol(
    response: ProtocolOfflineLeaseRefreshResponse,
) -> Result<OfflineLeaseRefreshResponse, RadishFlowControlPlaneClientError> {
    Ok(OfflineLeaseRefreshResponse {
        refreshed_at: response.refreshed_at,
        snapshot: entitlement_snapshot_from_protocol(response.snapshot)?,
        manifest_list: property_package_manifest_list_from_protocol(response.manifest_list)?,
    })
}

fn property_package_source_from_protocol(
    source: ProtocolPropertyPackageSource,
) -> PropertyPackageSource {
    match source {
        ProtocolPropertyPackageSource::Bundled => PropertyPackageSource::LocalBundled,
        ProtocolPropertyPackageSource::Download => PropertyPackageSource::RemoteDerivedPackage,
        ProtocolPropertyPackageSource::RemoteEval => PropertyPackageSource::RemoteEvaluationService,
    }
}

fn property_package_classification_from_protocol(
    classification: ProtocolPropertyPackageClassification,
) -> PropertyPackageClassification {
    match classification {
        ProtocolPropertyPackageClassification::Derived => PropertyPackageClassification::Derived,
        ProtocolPropertyPackageClassification::RemoteOnly => {
            PropertyPackageClassification::RemoteOnly
        }
    }
}

fn validate_protocol_schema_version(
    version: u32,
    entity_name: &str,
) -> Result<(), RadishFlowControlPlaneClientError> {
    if version != RADISHFLOW_CONTROL_PLANE_SCHEMA_VERSION {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            format!("unsupported {entity_name} schema version `{version}`"),
        ));
    }

    Ok(())
}

mod utils;

#[cfg(test)]
mod tests;
