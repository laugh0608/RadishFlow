use std::time::{Duration, SystemTime};

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

fn validate_access_token(access_token: &str) -> Result<(), RadishFlowControlPlaneClientError> {
    if access_token.trim().is_empty() {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            "control plane access token must not be empty",
        ));
    }

    Ok(())
}

fn validate_package_id(package_id: &str) -> Result<(), RadishFlowControlPlaneClientError> {
    if package_id.trim().is_empty() {
        return Err(RadishFlowControlPlaneClientError::invalid_response(
            "property package id must not be empty",
        ));
    }

    Ok(())
}

fn normalize_base_url(base_url: String) -> RfResult<String> {
    let normalized = base_url.trim().trim_end_matches('/').to_string();
    if normalized.is_empty() {
        return Err(RfError::invalid_input(
            "radishflow control plane base_url must not be empty",
        ));
    }

    Ok(normalized)
}

fn percent_encode_path_segment(segment: &str) -> String {
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

fn map_http_transport_error(
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

fn map_reqwest_transport_error(error: reqwest::Error) -> RadishFlowControlPlaneHttpTransportError {
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

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rf_ui::{OfflineLeaseRefreshRequest, PropertyPackageLeaseRequest};

    use crate::{
        HttpRadishFlowControlPlaneClient, RadishFlowControlPlaneClient,
        RadishFlowControlPlaneClientErrorKind, RadishFlowControlPlaneEndpoints,
        RadishFlowControlPlaneHttpRequest, RadishFlowControlPlaneHttpResponse,
        RadishFlowControlPlaneHttpTransport, RadishFlowControlPlaneHttpTransportError,
        ReqwestRadishFlowControlPlaneHttpTransport,
        ReqwestRadishFlowControlPlaneHttpTransportOptions,
    };

    #[test]
    fn entitlement_and_manifest_requests_use_bearer_token_and_json_accept() {
        let transport = ScriptedTransport::new(vec![
            Ok(RadishFlowControlPlaneHttpResponse::new(
                200,
                sample_entitlement_json(),
                timestamp(200),
            )),
            Ok(RadishFlowControlPlaneHttpResponse::new(
                200,
                sample_manifest_list_json(),
                timestamp(210),
            )),
        ]);
        let client = HttpRadishFlowControlPlaneClient::new(sample_endpoints(), &transport);

        let entitlement = client
            .fetch_entitlement_snapshot("access-token")
            .expect("expected entitlement fetch");
        let manifests = client
            .fetch_property_package_manifest_list("access-token")
            .expect("expected manifest fetch");

        assert_eq!(entitlement.value.subject_id, "user-123");
        assert_eq!(manifests.value.packages.len(), 1);
        assert_eq!(transport.call_count(), 2);
        assert_eq!(
            transport.requests()[0].url,
            "https://control.radish.local/api/radishflow/entitlements/current"
        );
        assert_eq!(transport.requests()[0].bearer_token, "access-token");
        assert_eq!(
            transport.requests()[1].url,
            "https://control.radish.local/api/radishflow/property-packages/manifest"
        );
        assert_eq!(
            transport.requests()[1].accept_content_types,
            vec!["application/json".to_string()]
        );
    }

    #[test]
    fn lease_request_serializes_runtime_request_body() {
        let transport = ScriptedTransport::new(vec![Ok(RadishFlowControlPlaneHttpResponse::new(
            200,
            sample_lease_grant_json(),
            timestamp(220),
        ))]);
        let client = HttpRadishFlowControlPlaneClient::new(sample_endpoints(), &transport);
        let mut request = PropertyPackageLeaseRequest::new("2026.03.1");
        request.current_hash = Some("sha256:current".to_string());
        request.installation_id = Some("studio-installation-001".to_string());

        let response = client
            .request_property_package_lease("access-token", "binary-hydrocarbon-lite-v1", &request)
            .expect("expected lease grant");

        assert_eq!(response.value.lease_id, "lease-1");
        assert_eq!(transport.call_count(), 1);
        assert_eq!(
            transport.requests()[0].content_type.as_deref(),
            Some("application/json")
        );
        assert_eq!(
            transport.requests()[0].body.as_deref(),
            Some(
                "{\"version\":\"2026.03.1\",\"currentHash\":\"sha256:current\",\"installationId\":\"studio-installation-001\"}"
            )
        );
    }

    #[test]
    fn offline_refresh_request_serializes_package_ids_and_maps_response() {
        let transport = ScriptedTransport::new(vec![Ok(RadishFlowControlPlaneHttpResponse::new(
            200,
            sample_offline_refresh_response_json(),
            timestamp(230),
        ))]);
        let client = HttpRadishFlowControlPlaneClient::new(sample_endpoints(), &transport);
        let request = OfflineLeaseRefreshRequest {
            package_ids: ["binary-hydrocarbon-lite-v1".to_string()]
                .into_iter()
                .collect(),
            current_offline_lease_expires_at: Some(timestamp(900)),
            installation_id: Some("studio-installation-001".to_string()),
        };

        let response = client
            .refresh_offline_leases("access-token", &request)
            .expect("expected offline refresh response");

        assert_eq!(response.value.snapshot.subject_id, "user-123");
        assert_eq!(
            transport.requests()[0].body.as_deref(),
            Some(
                "{\"packageIds\":[\"binary-hydrocarbon-lite-v1\"],\"currentOfflineLeaseExpiresAt\":\"1970-01-01T00:15:00Z\",\"installationId\":\"studio-installation-001\"}"
            )
        );
    }

    #[test]
    fn client_maps_http_statuses_into_existing_error_kinds() {
        let cases = [
            (401, RadishFlowControlPlaneClientErrorKind::Unauthorized),
            (403, RadishFlowControlPlaneClientErrorKind::Forbidden),
            (404, RadishFlowControlPlaneClientErrorKind::NotFound),
            (408, RadishFlowControlPlaneClientErrorKind::Timeout),
            (429, RadishFlowControlPlaneClientErrorKind::RateLimited),
            (
                503,
                RadishFlowControlPlaneClientErrorKind::ServiceUnavailable,
            ),
            (302, RadishFlowControlPlaneClientErrorKind::InvalidResponse),
        ];

        for (status_code, expected_kind) in cases {
            let transport = ScriptedTransport::new(vec![Ok(
                RadishFlowControlPlaneHttpResponse::new(status_code, "{}", timestamp(200)),
            )]);
            let client = HttpRadishFlowControlPlaneClient::new(sample_endpoints(), &transport);

            let error = client
                .fetch_entitlement_snapshot("access-token")
                .expect_err("expected mapped status error");

            assert_eq!(error.kind, expected_kind);
        }
    }

    #[test]
    fn reqwest_transport_sends_bearer_and_json_body() {
        let server = spawn_http_server(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}"
                .to_string(),
        );
        let transport = ReqwestRadishFlowControlPlaneHttpTransport::with_options(
            ReqwestRadishFlowControlPlaneHttpTransportOptions {
                request_timeout: Duration::from_secs(5),
                user_agent: "radishflow-test".to_string(),
            },
        )
        .expect("expected reqwest transport");
        let request = RadishFlowControlPlaneHttpRequest::new_post_json(
            server.url(),
            "access-token",
            "{\"hello\":\"world\"}",
        );

        let response = transport.send(&request).expect("expected transport send");
        let request_text = server.request_text();

        assert_eq!(response.status_code, 200);
        assert!(request_text.contains("post /control http/1.1"));
        assert!(request_text.contains("authorization: bearer access-token"));
        assert!(request_text.contains("content-type: application/json"));
        assert!(request_text.contains("accept: application/json"));
        assert!(request_text.contains("{\"hello\":\"world\"}"));
    }

    fn sample_endpoints() -> RadishFlowControlPlaneEndpoints {
        RadishFlowControlPlaneEndpoints::new("https://control.radish.local")
            .expect("expected control plane endpoints")
    }

    fn sample_entitlement_json() -> String {
        r#"{
  "schemaVersion": 1,
  "subjectId": "user-123",
  "tenantId": "tenant-1",
  "issuedAt": "1970-01-01T00:01:40Z",
  "expiresAt": "1970-01-01T00:08:20Z",
  "offlineLeaseExpiresAt": "1970-01-01T00:15:00Z",
  "features": ["desktop-login", "local-thermo-packages"],
  "allowedPackageIds": ["binary-hydrocarbon-lite-v1"]
}"#
        .to_string()
    }

    fn sample_manifest_list_json() -> String {
        r#"{
  "schemaVersion": 1,
  "generatedAt": "1970-01-01T00:03:20Z",
  "packages": [
    {
      "schemaVersion": 1,
      "packageId": "binary-hydrocarbon-lite-v1",
      "version": "2026.03.1",
      "classification": "derived",
      "source": "download",
      "hash": "sha256:pkg-1",
      "sizeBytes": 1024,
      "componentIds": ["methane", "ethane"],
      "leaseRequired": true,
      "expiresAt": "1970-01-01T00:15:00Z"
    }
  ]
}"#
        .to_string()
    }

    fn sample_lease_grant_json() -> String {
        r#"{
  "packageId": "binary-hydrocarbon-lite-v1",
  "version": "2026.03.1",
  "leaseId": "lease-1",
  "downloadUrl": "https://assets.radish.local/leases/lease-1/download",
  "hash": "sha256:pkg-1",
  "sizeBytes": 1024,
  "expiresAt": "1970-01-01T00:04:10Z"
}"#
        .to_string()
    }

    fn sample_offline_refresh_response_json() -> String {
        format!(
            r#"{{
  "refreshedAt": "1970-01-01T00:03:30Z",
  "snapshot": {},
  "manifestList": {}
}}"#,
            sample_entitlement_json(),
            sample_manifest_list_json()
        )
    }

    fn timestamp(seconds: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    struct ScriptedTransport {
        responses: RefCell<
            Vec<
                Result<
                    RadishFlowControlPlaneHttpResponse,
                    RadishFlowControlPlaneHttpTransportError,
                >,
            >,
        >,
        call_count: Cell<u32>,
        requests: RefCell<Vec<RadishFlowControlPlaneHttpRequest>>,
    }

    impl ScriptedTransport {
        fn new(
            responses: Vec<
                Result<
                    RadishFlowControlPlaneHttpResponse,
                    RadishFlowControlPlaneHttpTransportError,
                >,
            >,
        ) -> Self {
            Self {
                responses: RefCell::new(responses),
                call_count: Cell::new(0),
                requests: RefCell::new(Vec::new()),
            }
        }

        fn call_count(&self) -> u32 {
            self.call_count.get()
        }

        fn requests(&self) -> Vec<RadishFlowControlPlaneHttpRequest> {
            self.requests.borrow().clone()
        }
    }

    impl RadishFlowControlPlaneHttpTransport for ScriptedTransport {
        fn send(
            &self,
            request: &RadishFlowControlPlaneHttpRequest,
        ) -> Result<RadishFlowControlPlaneHttpResponse, RadishFlowControlPlaneHttpTransportError>
        {
            self.call_count.set(self.call_count.get() + 1);
            self.requests.borrow_mut().push(request.clone());
            self.responses.borrow_mut().remove(0)
        }
    }

    struct LocalHttpTestServer {
        address: std::net::SocketAddr,
        request_text: std::sync::Arc<std::sync::Mutex<Option<String>>>,
        thread: Option<thread::JoinHandle<()>>,
    }

    impl LocalHttpTestServer {
        fn url(&self) -> String {
            format!("http://{}/control", self.address)
        }

        fn request_text(mut self) -> String {
            if let Some(thread) = self.thread.take() {
                thread.join().expect("expected local http server join");
            }

            self.request_text
                .lock()
                .expect("expected request text lock")
                .clone()
                .expect("expected captured request")
                .to_ascii_lowercase()
        }
    }

    fn spawn_http_server(response_text: String) -> LocalHttpTestServer {
        let listener = TcpListener::bind("127.0.0.1:0").expect("expected local listener");
        let address = listener.local_addr().expect("expected local address");
        let request_text = std::sync::Arc::new(std::sync::Mutex::new(None));
        let request_text_for_thread = request_text.clone();

        let thread = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("expected client connection");
            let mut buffer = Vec::new();
            let mut chunk = [0u8; 1024];
            loop {
                let read = stream.read(&mut chunk).expect("expected request read");
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
                if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            *request_text_for_thread
                .lock()
                .expect("expected request text lock") =
                Some(String::from_utf8_lossy(&buffer).to_string());
            stream
                .write_all(response_text.as_bytes())
                .expect("expected response write");
            stream.flush().expect("expected response flush");
        });

        LocalHttpTestServer {
            address,
            request_text,
            thread: Some(thread),
        }
    }
}
