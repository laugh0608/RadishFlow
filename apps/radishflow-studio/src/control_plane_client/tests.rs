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
    ReqwestRadishFlowControlPlaneHttpTransport, ReqwestRadishFlowControlPlaneHttpTransportOptions,
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
        let transport = ScriptedTransport::new(vec![Ok(RadishFlowControlPlaneHttpResponse::new(
            status_code,
            "{}",
            timestamp(200),
        ))]);
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
        Vec<Result<RadishFlowControlPlaneHttpResponse, RadishFlowControlPlaneHttpTransportError>>,
    >,
    call_count: Cell<u32>,
    requests: RefCell<Vec<RadishFlowControlPlaneHttpRequest>>,
}

impl ScriptedTransport {
    fn new(
        responses: Vec<
            Result<RadishFlowControlPlaneHttpResponse, RadishFlowControlPlaneHttpTransportError>,
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
    ) -> Result<RadishFlowControlPlaneHttpResponse, RadishFlowControlPlaneHttpTransportError> {
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
