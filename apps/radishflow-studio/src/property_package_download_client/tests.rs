use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rf_store::{
    StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache,
    property_package_payload_integrity, read_property_package_payload,
};
use rf_types::ComponentId;
use rf_ui::{PropertyPackageLeaseGrant, PropertyPackageManifest, PropertyPackageSource};

use crate::{
    HttpPropertyPackageDownloadFetcher, PropertyPackageDownloadFetchError,
    PropertyPackageDownloadFetchErrorKind, PropertyPackageDownloadFetcher,
    PropertyPackageDownloadHttpRequest, PropertyPackageDownloadHttpResponse,
    PropertyPackageDownloadHttpTransport, PropertyPackageDownloadHttpTransportError,
    PropertyPackageDownloadResponse, PropertyPackageDownloadRetryPolicy,
    ReqwestPropertyPackageDownloadHttpTransport,
    ReqwestPropertyPackageDownloadHttpTransportOptions, download_property_package_to_cache,
    download_property_package_to_cache_with_retry_policy, parse_property_package_download_json,
};

struct StaticDownloadFetcher {
    response: PropertyPackageDownloadResponse,
}

impl PropertyPackageDownloadFetcher for StaticDownloadFetcher {
    fn fetch_download(
        &self,
        _lease_grant: &PropertyPackageLeaseGrant,
    ) -> Result<PropertyPackageDownloadResponse, PropertyPackageDownloadFetchError> {
        Ok(self.response.clone())
    }
}

struct ScriptedDownloadFetcher {
    responses: RefCell<Vec<Result<PropertyPackageDownloadResponse, PropertyPackageDownloadFetchError>>>,
    call_count: Cell<u32>,
}

impl ScriptedDownloadFetcher {
    fn new(
        responses: Vec<Result<PropertyPackageDownloadResponse, PropertyPackageDownloadFetchError>>,
    ) -> Self {
        Self {
            responses: RefCell::new(responses),
            call_count: Cell::new(0),
        }
    }

    fn call_count(&self) -> u32 {
        self.call_count.get()
    }
}

impl PropertyPackageDownloadFetcher for ScriptedDownloadFetcher {
    fn fetch_download(
        &self,
        _lease_grant: &PropertyPackageLeaseGrant,
    ) -> Result<PropertyPackageDownloadResponse, PropertyPackageDownloadFetchError> {
        self.call_count.set(self.call_count.get() + 1);
        self.responses.borrow_mut().remove(0)
    }
}

struct ScriptedHttpTransport {
    responses: RefCell<
        Vec<
            Result<
                PropertyPackageDownloadHttpResponse,
                PropertyPackageDownloadHttpTransportError,
            >,
        >,
    >,
    call_count: Cell<u32>,
    requests: RefCell<Vec<PropertyPackageDownloadHttpRequest>>,
}

impl ScriptedHttpTransport {
    fn new(
        responses: Vec<
            Result<
                PropertyPackageDownloadHttpResponse,
                PropertyPackageDownloadHttpTransportError,
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

    fn requests(&self) -> Vec<PropertyPackageDownloadHttpRequest> {
        self.requests.borrow().clone()
    }
}

impl PropertyPackageDownloadHttpTransport for ScriptedHttpTransport {
    fn send(
        &self,
        request: &PropertyPackageDownloadHttpRequest,
    ) -> Result<PropertyPackageDownloadHttpResponse, PropertyPackageDownloadHttpTransportError>
    {
        self.call_count.set(self.call_count.get() + 1);
        self.requests.borrow_mut().push(request.clone());
        self.responses.borrow_mut().remove(0)
    }
}

#[test]
fn download_property_package_to_cache_fetches_response_and_persists_assets() {
    let root = unique_temp_path("download-fetcher-success");
    let mut index = sample_auth_cache_index();
    let download = parse_property_package_download_json(&sample_download_json())
        .expect("expected sample download");
    let payload = download
        .to_stored_payload()
        .expect("expected sample payload");
    let integrity = property_package_payload_integrity(&payload).expect("expected payload integrity");
    let manifest = sample_manifest(&integrity.hash, integrity.size_bytes);
    let lease_grant = sample_lease_grant(&integrity.hash, integrity.size_bytes);
    let fetcher = StaticDownloadFetcher {
        response: PropertyPackageDownloadResponse::new(sample_download_json(), timestamp(200)),
    };

    download_property_package_to_cache(&root, &mut index, &manifest, &lease_grant, &fetcher)
        .expect("expected cached download");

    let cached_record = &index.property_packages[0];
    let payload = read_property_package_payload(
        cached_record
            .payload_path_under(&root)
            .expect("expected payload path"),
    )
    .expect("expected payload read");

    assert_eq!(payload.package_id, "binary-hydrocarbon-lite-v1");
    assert_eq!(payload.components.len(), 2);

    fs::remove_dir_all(&root).expect("expected temp dir cleanup");
}

#[test]
fn download_property_package_to_cache_rejects_hash_mismatch_before_updating_index() {
    let root = unique_temp_path("download-fetcher-mismatch");
    let mut index = sample_auth_cache_index();
    let download = parse_property_package_download_json(&sample_download_json())
        .expect("expected sample download");
    let payload = download
        .to_stored_payload()
        .expect("expected sample payload");
    let integrity = property_package_payload_integrity(&payload).expect("expected payload integrity");
    let manifest = sample_manifest(&integrity.hash, integrity.size_bytes);
    let lease_grant = sample_lease_grant(&integrity.hash, integrity.size_bytes);
    let fetcher = StaticDownloadFetcher {
        response: PropertyPackageDownloadResponse::new(
            sample_download_json().replace("\"Methane\"", "\"Methane Modified\""),
            timestamp(200),
        ),
    };

    let error = download_property_package_to_cache(
        &root,
        &mut index,
        &manifest,
        &lease_grant,
        &fetcher,
    )
    .expect_err("expected hash mismatch");

    assert_eq!(error.code().as_str(), "invalid_input");
    assert!(error.message().contains("payload hash"));
    assert!(index.property_packages.is_empty());
    fs::remove_dir_all(&root).ok();
}

#[test]
fn download_property_package_to_cache_retries_retryable_errors_before_success() {
    let root = unique_temp_path("download-fetcher-retryable");
    let mut index = sample_auth_cache_index();
    let download = parse_property_package_download_json(&sample_download_json())
        .expect("expected sample download");
    let payload = download
        .to_stored_payload()
        .expect("expected sample payload");
    let integrity = property_package_payload_integrity(&payload).expect("expected payload integrity");
    let manifest = sample_manifest(&integrity.hash, integrity.size_bytes);
    let lease_grant = sample_lease_grant(&integrity.hash, integrity.size_bytes);
    let fetcher = ScriptedDownloadFetcher::new(vec![
        Err(PropertyPackageDownloadFetchError::timeout("adapter timed out")),
        Err(PropertyPackageDownloadFetchError::service_unavailable(
            "asset delivery is warming up",
        )),
        Ok(PropertyPackageDownloadResponse::new(
            sample_download_json(),
            timestamp(200),
        )),
    ]);

    download_property_package_to_cache(&root, &mut index, &manifest, &lease_grant, &fetcher)
        .expect("expected cached download after retries");

    assert_eq!(fetcher.call_count(), 3);
    assert_eq!(index.property_packages.len(), 1);
    fs::remove_dir_all(&root).ok();
}

#[test]
fn download_property_package_to_cache_does_not_retry_non_retryable_errors() {
    let root = unique_temp_path("download-fetcher-non-retryable");
    let mut index = sample_auth_cache_index();
    let download = parse_property_package_download_json(&sample_download_json())
        .expect("expected sample download");
    let payload = download
        .to_stored_payload()
        .expect("expected sample payload");
    let integrity = property_package_payload_integrity(&payload).expect("expected payload integrity");
    let manifest = sample_manifest(&integrity.hash, integrity.size_bytes);
    let lease_grant = sample_lease_grant(&integrity.hash, integrity.size_bytes);
    let fetcher = ScriptedDownloadFetcher::new(vec![
        Err(PropertyPackageDownloadFetchError::unauthorized(
            "lease is no longer valid",
        )),
        Ok(PropertyPackageDownloadResponse::new(
            sample_download_json(),
            timestamp(200),
        )),
    ]);

    let error = download_property_package_to_cache(
        &root,
        &mut index,
        &manifest,
        &lease_grant,
        &fetcher,
    )
    .expect_err("expected non-retryable fetch failure");

    assert_eq!(fetcher.call_count(), 1);
    assert_eq!(error.code().as_str(), "invalid_input");
    assert!(error.message().contains("non-retryable unauthorized error"));
    assert!(index.property_packages.is_empty());
    fs::remove_dir_all(&root).ok();
}

#[test]
fn download_property_package_to_cache_reports_retry_exhaustion() {
    let root = unique_temp_path("download-fetcher-exhausted");
    let mut index = sample_auth_cache_index();
    let download = parse_property_package_download_json(&sample_download_json())
        .expect("expected sample download");
    let payload = download
        .to_stored_payload()
        .expect("expected sample payload");
    let integrity = property_package_payload_integrity(&payload).expect("expected payload integrity");
    let manifest = sample_manifest(&integrity.hash, integrity.size_bytes);
    let lease_grant = sample_lease_grant(&integrity.hash, integrity.size_bytes);
    let fetcher = ScriptedDownloadFetcher::new(vec![
        Err(PropertyPackageDownloadFetchError::rate_limited("retry later")),
        Err(PropertyPackageDownloadFetchError::rate_limited(
            "retry later again",
        )),
    ]);
    let retry_policy =
        PropertyPackageDownloadRetryPolicy::new(2).expect("expected retry policy");

    let error = download_property_package_to_cache_with_retry_policy(
        &root,
        &mut index,
        &manifest,
        &lease_grant,
        &fetcher,
        retry_policy,
    )
    .expect_err("expected retry exhaustion");

    assert_eq!(fetcher.call_count(), 2);
    assert!(error.message().contains("exhausted 2 attempts"));
    assert!(index.property_packages.is_empty());
    fs::remove_dir_all(&root).ok();
}

#[test]
fn http_fetcher_builds_request_from_lease_grant() {
    let transport = ScriptedHttpTransport::new(vec![Ok(PropertyPackageDownloadHttpResponse::new(
        200,
        sample_download_json(),
        timestamp(200),
    ))]);
    let fetcher = HttpPropertyPackageDownloadFetcher::new(&transport);

    let response = fetcher
        .fetch_download(&sample_lease_grant("sha256:test", 1))
        .expect("expected http fetch success");

    assert_eq!(response.contents, sample_download_json());
    assert_eq!(transport.call_count(), 1);
    assert_eq!(transport.requests().len(), 1);
    assert_eq!(
        transport.requests()[0].url,
        "https://assets.radish.local/lease-1"
    );
    assert_eq!(
        transport.requests()[0].accept_content_types,
        vec!["application/json".to_string()]
    );
}

#[test]
fn http_fetcher_maps_http_statuses_into_existing_failure_categories() {
    let cases = [
        (401, PropertyPackageDownloadFetchErrorKind::Unauthorized),
        (403, PropertyPackageDownloadFetchErrorKind::Forbidden),
        (404, PropertyPackageDownloadFetchErrorKind::NotFound),
        (408, PropertyPackageDownloadFetchErrorKind::Timeout),
        (429, PropertyPackageDownloadFetchErrorKind::RateLimited),
        (503, PropertyPackageDownloadFetchErrorKind::ServiceUnavailable),
        (504, PropertyPackageDownloadFetchErrorKind::Timeout),
        (302, PropertyPackageDownloadFetchErrorKind::InvalidResponse),
    ];

    for (status_code, expected_kind) in cases {
        let transport = ScriptedHttpTransport::new(vec![Ok(
            PropertyPackageDownloadHttpResponse::new(status_code, "{}", timestamp(200)),
        )]);
        let fetcher = HttpPropertyPackageDownloadFetcher::new(&transport);

        let error = fetcher
            .fetch_download(&sample_lease_grant("sha256:test", 1))
            .expect_err("expected mapped http failure");

        assert_eq!(error.kind, expected_kind);
    }
}

#[test]
fn http_fetcher_maps_transport_errors_into_existing_failure_categories() {
    let cases = [
        (
            PropertyPackageDownloadHttpTransportError::timeout("timed out"),
            PropertyPackageDownloadFetchErrorKind::Timeout,
        ),
        (
            PropertyPackageDownloadHttpTransportError::connection_unavailable("offline"),
            PropertyPackageDownloadFetchErrorKind::ConnectionUnavailable,
        ),
        (
            PropertyPackageDownloadHttpTransportError::other_transient("proxy reset"),
            PropertyPackageDownloadFetchErrorKind::OtherTransient,
        ),
        (
            PropertyPackageDownloadHttpTransportError::other_permanent("bad TLS config"),
            PropertyPackageDownloadFetchErrorKind::OtherPermanent,
        ),
    ];

    for (transport_error, expected_kind) in cases {
        let transport = ScriptedHttpTransport::new(vec![Err(transport_error)]);
        let fetcher = HttpPropertyPackageDownloadFetcher::new(&transport);

        let error = fetcher
            .fetch_download(&sample_lease_grant("sha256:test", 1))
            .expect_err("expected mapped transport failure");

        assert_eq!(error.kind, expected_kind);
    }
}

#[test]
fn http_fetcher_rejects_success_without_json_content_type() {
    let mut response =
        PropertyPackageDownloadHttpResponse::new(200, sample_download_json(), timestamp(200));
    response.content_type = Some("application/octet-stream".to_string());
    let transport = ScriptedHttpTransport::new(vec![Ok(response)]);
    let fetcher = HttpPropertyPackageDownloadFetcher::new(&transport);

    let error = fetcher
        .fetch_download(&sample_lease_grant("sha256:test", 1))
        .expect_err("expected invalid response content type");

    assert_eq!(
        error.kind,
        PropertyPackageDownloadFetchErrorKind::InvalidResponse
    );
}

#[test]
fn reqwest_transport_fetches_local_http_response() {
    let server = spawn_http_server(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}"
            .to_string(),
    );
    let transport = ReqwestPropertyPackageDownloadHttpTransport::with_options(
        ReqwestPropertyPackageDownloadHttpTransportOptions {
            request_timeout: Duration::from_secs(5),
            user_agent: "radishflow-test".to_string(),
        },
    )
    .expect("expected reqwest transport");

    let response = transport
        .send(&PropertyPackageDownloadHttpRequest::new(server.url()))
        .expect("expected reqwest fetch");
    let request_text = server.request_text();

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, "{}");
    assert_eq!(response.content_type, Some("application/json".to_string()));
    assert!(request_text.contains("get /download http/1.1"));
    assert!(request_text.contains("accept: application/json"));
    assert!(request_text.contains("user-agent: radishflow-test"));
}

#[test]
fn reqwest_transport_maps_connection_errors() {
    let closed_listener = TcpListener::bind("127.0.0.1:0").expect("expected free port");
    let address = closed_listener.local_addr().expect("expected local address");
    drop(closed_listener);

    let transport = ReqwestPropertyPackageDownloadHttpTransport::with_options(
        ReqwestPropertyPackageDownloadHttpTransportOptions {
            request_timeout: Duration::from_millis(200),
            user_agent: "radishflow-test".to_string(),
        },
    )
    .expect("expected reqwest transport");

    let error = transport
        .send(&PropertyPackageDownloadHttpRequest::new(format!(
            "http://{address}/download"
        )))
        .expect_err("expected connection error");

    assert!(matches!(
        error.kind,
        super::PropertyPackageDownloadHttpTransportErrorKind::ConnectionUnavailable
            | super::PropertyPackageDownloadHttpTransportErrorKind::Timeout
    ));
}

fn sample_auth_cache_index() -> StoredAuthCacheIndex {
    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user-123",
        StoredCredentialReference::new("radishflow-studio", "user-credential"),
    );
    index.entitlement = Some(StoredEntitlementCache {
        subject_id: "user-123".to_string(),
        tenant_id: Some("tenant-1".to_string()),
        synced_at: timestamp(100),
        issued_at: timestamp(90),
        expires_at: timestamp(500),
        offline_lease_expires_at: Some(timestamp(900)),
        feature_keys: BTreeSet::from(["desktop-login".to_string()]),
        allowed_package_ids: BTreeSet::from(["binary-hydrocarbon-lite-v1".to_string()]),
    });
    index
}

fn sample_manifest(hash: &str, size_bytes: u64) -> PropertyPackageManifest {
    let mut manifest = PropertyPackageManifest::new(
        "binary-hydrocarbon-lite-v1",
        "2026.03.1",
        PropertyPackageSource::RemoteDerivedPackage,
    );
    manifest.hash = hash.to_string();
    manifest.size_bytes = size_bytes;
    manifest.component_ids = vec![ComponentId::new("methane"), ComponentId::new("ethane")];
    manifest
}

fn sample_lease_grant(hash: &str, size_bytes: u64) -> PropertyPackageLeaseGrant {
    PropertyPackageLeaseGrant {
        package_id: "binary-hydrocarbon-lite-v1".to_string(),
        version: "2026.03.1".to_string(),
        lease_id: "lease-1".to_string(),
        download_url: "https://assets.radish.local/lease-1".to_string(),
        hash: hash.to_string(),
        size_bytes,
        expires_at: timestamp(210),
    }
}

fn sample_download_json() -> String {
    fs::read_to_string(sample_download_path()).expect("expected sample download json")
}

fn sample_download_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../examples/sample-components/property-packages/binary-hydrocarbon-lite-v1/download.json",
    )
}

fn timestamp(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

fn unique_temp_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected time after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("radishflow-{name}-{unique}"))
}

struct LocalHttpTestServer {
    address: std::net::SocketAddr,
    request_text: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    thread: Option<thread::JoinHandle<()>>,
}

impl LocalHttpTestServer {
    fn url(&self) -> String {
        format!("http://{}/download", self.address)
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
            Some(String::from_utf8_lossy(&buffer).to_ascii_lowercase());
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
