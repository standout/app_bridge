//! Per-connection TLS identity for outbound HTTP (mTLS and extra CAs).
//!
//! The platform puts certificate material in `connection.serialized-data.tls`
//! (from connection-config `runtime.tls`). Canonical keys inside that object
//! (platform-normalized before WASM):
//! - `client_cert` + `client_key`: certificate PEM and unencrypted PKCS#8 key
//! - `client_p12` + optional `client_p12_password`: PKCS#12 / PFX (base64 or raw)
//! - `ca_cert`: optional extra CA PEM (single cert or bundle; all roots are trusted)
//!
//! Combined PEMs, encrypted keys, PKCS#1, and SEC1 are handled by the platform
//! (Integrationer), not here. Only `serialized-data.tls` is read.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::blocking::Client;
use reqwest::{Certificate, Identity};
use serde_json::Value;

const CLIENT_CERT: &str = "client_cert";
const CLIENT_KEY: &str = "client_key";
const CLIENT_P12: &str = "client_p12";
const CLIENT_P12_PASSWORD: &str = "client_p12_password";
const CA_CERT: &str = "ca_cert";

/// Builds a reqwest client from connection serialized JSON when TLS material is present.
///
/// Returns `Ok(None)` when no TLS keys are set (caller should use the default client).
pub fn client_from_serialized_data(serialized_data: &str) -> Result<Option<Client>, String> {
    let tls = TlsMaterial::parse(serialized_data)?;
    tls.build_client()
}

struct TlsMaterial {
    identity: Option<Identity>,
    ca_certs: Vec<Certificate>,
}

impl TlsMaterial {
    fn parse(serialized_data: &str) -> Result<Self, String> {
        let trimmed = serialized_data.trim();
        if trimmed.is_empty() {
            return Ok(Self {
                identity: None,
                ca_certs: Vec::new(),
            });
        }

        let value: Value = serde_json::from_str(trimmed).unwrap_or(Value::Null);
        let object = match value.as_object() {
            Some(object) => object,
            None => {
                return Ok(Self {
                    identity: None,
                    ca_certs: Vec::new(),
                });
            }
        };

        let tls_object = match object.get("tls") {
            None | Some(Value::Null) => {
                return Ok(Self {
                    identity: None,
                    ca_certs: Vec::new(),
                });
            }
            Some(value) => value
                .as_object()
                .ok_or_else(|| "serialized-data.tls must be an object".to_string())?,
        };

        let identity = identity_from_object(tls_object)?;
        let ca_certs = match tls_object.get(CA_CERT) {
            Some(value) if !value.is_null() => {
                let bytes = material_from_value(value).ok_or_else(|| {
                    "ca_cert is present but empty or not a PEM/base64 string".to_string()
                })?;
                ca_certificates_from_material(&bytes)?
            }
            _ => Vec::new(),
        };

        Ok(Self { identity, ca_certs })
    }

    fn build_client(self) -> Result<Option<Client>, String> {
        if self.identity.is_none() && self.ca_certs.is_empty() {
            return Ok(None);
        }

        let mut builder = Client::builder();
        if let Some(identity) = self.identity {
            builder = builder.identity(identity);
        }
        for cert in self.ca_certs {
            builder = builder.add_root_certificate(cert);
        }
        builder
            .build()
            .map(Some)
            .map_err(|err| format!("Failed to build TLS HTTP client: {err}"))
    }
}

#[cfg(test)]
pub(crate) fn test_pem_connection_json() -> String {
    serde_json::json!({
        "tls": {
            "client_cert": tests::TEST_CERT,
            "client_key": tests::TEST_KEY,
        }
    })
    .to_string()
}

fn identity_from_object(
    object: &serde_json::Map<String, Value>,
) -> Result<Option<Identity>, String> {
    if let Some(p12_value) = object.get(CLIENT_P12).filter(|value| !value.is_null()) {
        let der = material_from_value(p12_value)
            .ok_or_else(|| "client_p12 is present but empty".to_string())?;
        let password = object
            .get(CLIENT_P12_PASSWORD)
            .and_then(Value::as_str)
            .unwrap_or("");
        return Identity::from_pkcs12_der(&der, password)
            .map(Some)
            .map_err(|err| format!("Invalid client_p12 PKCS#12 identity: {err}"));
    }

    let cert = object
        .get(CLIENT_CERT)
        .filter(|value| !value.is_null())
        .and_then(material_from_value);
    let key = object
        .get(CLIENT_KEY)
        .filter(|value| !value.is_null())
        .and_then(material_from_value);

    match (cert, key) {
        (None, None) => Ok(None),
        (None, Some(_)) => {
            Err("client_key is set without client_cert; provide both PEM fields".to_string())
        }
        (Some(_), None) => Err(
            "client_key is required; platform must split combined PEM before WASM".to_string(),
        ),
        (Some(cert), Some(key)) => {
            if pem_is_encrypted(&key) {
                return Err(
                    "encrypted PEM private key; platform must decrypt before WASM".to_string(),
                );
            }
            Identity::from_pkcs8_pem(&cert, &key)
                .map(Some)
                .map_err(|err| format!("Invalid client_cert/client_key PEM identity: {err}"))
        }
    }
}

fn pem_is_encrypted(pem: &[u8]) -> bool {
    std::str::from_utf8(pem).is_ok_and(|text| {
        text.contains("ENCRYPTED PRIVATE KEY") || text.contains("Proc-Type: 4,ENCRYPTED")
    })
}

fn ca_certificates_from_material(bytes: &[u8]) -> Result<Vec<Certificate>, String> {
    let certs = Certificate::from_pem_bundle(bytes)
        .map_err(|err| format!("Invalid ca_cert PEM: {err}"))?;
    if certs.is_empty() {
        return Err("ca_cert has no CERTIFICATE PEM block".to_string());
    }
    Ok(certs)
}

fn material_from_value(value: &Value) -> Option<Vec<u8>> {
    match value {
        Value::String(text) => decode_material(text),
        Value::Object(map) => map
            .get("base64")
            .and_then(Value::as_str)
            .and_then(decode_material)
            .or_else(|| {
                map.get("pem")
                    .and_then(Value::as_str)
                    .and_then(decode_material)
            }),
        _ => None,
    }
}

fn decode_material(input: &str) -> Option<Vec<u8>> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.contains("-----BEGIN") {
        return Some(trimmed.as_bytes().to_vec());
    }
    if let Some(data_uri) = trimmed.strip_prefix("data:") {
        if let Some((_, data)) = data_uri.split_once(',') {
            return BASE64
                .decode(data.trim())
                .ok()
                .filter(|bytes| !bytes.is_empty());
        }
    }

    let compact: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
    match BASE64.decode(&compact) {
        Ok(bytes) if !bytes.is_empty() => Some(bytes),
        _ => Some(trimmed.as_bytes().to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) const TEST_CERT: &str = "-----BEGIN CERTIFICATE-----\n\
MIIDHzCCAgegAwIBAgIUO13GWglOjr4NUOGsRzrRUo+e3MwwDQYJKoZIhvcNAQEL\n\
BQAwHzEdMBsGA1UEAwwUYXBwLWJyaWRnZS1tdGxzLXRlc3QwHhcNMjYxMDA5MTI0\n\
ODIwWhcNMzYxMDA2MTI0ODIwWjAfMR0wGwYDVQQDDBRhcHAtYnJpZGdlLW10bHMt\n\
dGVzdDCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBANzA+L/tjF6TDDFY\n\
Oo6bfJBlu9LWzBqJv1OGtW0fyRb0m7kjxV7bz+wuqTWXIfE+o8bQ9nxoS0an+C0V\n\
1eZt8ueSjnrOXLeuHg/BwqytQSVUtpYlvX8Ih5XwPBQbwT7jXm/4oYYONyZLHVMG\n\
r+Xsr4lQaHHXmksWdiY0etH7cYpUJbf2h/A+GONVAdLrqkc7iACP34gGKgUC29Lf\n\
Gr4grD8kTP8uamG4T3/gMJtzuGgK96AoTzsXQ9YRkSTrm/X9Lx8OGXCVxBpOjbKc\n\
Nw3Ij+aRuUiUHNp+ePgDN65s42B9QNOk0540kMl2bV8rHi6RxZx7liwx0elcrlRs\n\
g7AfkscCAwEAAaNTMFEwHQYDVR0OBBYEFCNDTH9OjceLaiDGZhJFbW3uzobuMB8G\n\
A1UdIwQYMBaAFCNDTH9OjceLaiDGZhJFbW3uzobuMA8GA1UdEwEB/wQFMAMBAf8w\n\
DQYJKoZIhvcNAQELBQADggEBAA2vlnO4pNIVoSQLIGVATpVa9YPpy2G/2aM3A0ZH\n\
2RUDUjqkYLfHMy6urRhfAniHnEzrQDZ/HYc7T+sp9vNBTJPr6yBZg+IknkI5g7u5\n\
pgxMjyJhaCB11bf03zAVljY3+HugrJCtpjuz8Ydlj4vIXpFHl+ZUWT4idb6TjUQT\n\
crc8SwxDzxaNKVFSULZioVd3T7KreIAnyzZ71Zm0h7I25/+zGqSIu9bXsVGwnnXo\n\
Z5NMpHajzsHTZ/63A1Y8wTYR1300jAg6U/BfwWkwS61xu5tkFIxoP4cP6yQ0G+6m\n\
L79K80TLdO7Wnd5bCc4nHWvD1i/sGWENoJ+t7J7oe+2kDVs=\n\
-----END CERTIFICATE-----";

    pub(crate) const TEST_KEY: &str = "-----BEGIN PRIVATE KEY-----\n\
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDcwPi/7Yxekwwx\n\
WDqOm3yQZbvS1swaib9ThrVtH8kW9Ju5I8Ve28/sLqk1lyHxPqPG0PZ8aEtGp/gt\n\
FdXmbfLnko56zly3rh4PwcKsrUElVLaWJb1/CIeV8DwUG8E+415v+KGGDjcmSx1T\n\
Bq/l7K+JUGhx15pLFnYmNHrR+3GKVCW39ofwPhjjVQHS66pHO4gAj9+IBioFAtvS\n\
3xq+IKw/JEz/LmphuE9/4DCbc7hoCvegKE87F0PWEZEk65v1/S8fDhlwlcQaTo2y\n\
nDcNyI/mkblIlBzafnj4AzeubONgfUDTpNOeNJDJdm1fKx4ukcWce5YsMdHpXK5U\n\
bIOwH5LHAgMBAAECggEAErtt6udkQ7re9tRMSmvyQtSHKXR1pwOCUJBRQTHIdWIL\n\
UXIHc62xCT3KY6mampHcrD3GH2IVj9N9W+IK9RM7tfUkkaeE4ZyXoMb/LHpFLr7v\n\
VsgGukqwgAN92Eekpcqoaar46zBgnfy+FRyvp2c8Bqm5OykhnCE1+bb2pxkwQMhh\n\
HjhBVd+NKP+yUBq4gJ1KFq24ELrM9lc9+O4qpN2LA//YogrgwFvDFEfdR4rap1AW\n\
Yppn97kqahUT9duKcqqSefJcyEXsT8G5QB0wSL2zZO4M0cBKXapqtTG+mGBBLThE\n\
yF82DtOCpecCAM3gXgjS2858AEFfBaT2nueJlYUK9QKBgQD5Wj5zcGpeBkfZohTe\n\
qSMaqO/w7I842xjgWNlXXDQfBSBZmFoLNH3WfxZbWZ7mUz4J9sD8lJ4c0JAPF+eO\n\
VQJhTt5x7VFz4qgf20Y81pC5ucdWMr9XKeBPcX5fCJH98uoKuiY92QMgcBuVZT/r\n\
jW/aXkwUaBfuaP6psI5yLLQUCwKBgQDio4zQT2PCcvNlApy4HVdz9wRV0w5mFwnc\n\
iMEnjsCGriQtApNltdEexIxp9RRkPNS04s8vwDz69L4MOyYmZpHnAg43PwNBpa0e\n\
ctEbO426mkDN8C2oVIms6lJfX+3tuwdoTtj7lE9gjk8pxWZp9UfXEQqmjRTliKtX\n\
mzK9akmVtQKBgQCYV7JgBQZ3e47aQFRP1TZFS4GMyHqYWgPQzXe8D/EoIpGpK5OX\n\
1PIs4mheV1rljWYR4Ff3+TTAlgvtrMUkCHvzx6PQJxXkn03oaKd48/5n+kD3QPn1\n\
3vfUkZqLpUjMxmSHOUOz9aobX2L622k6GKeF2qbQsOguGKe+NMhSqU08EQKBgB25\n\
/fpEr30Nvsb89Llhf9sCB1uKd1UEqJJ4jX02yCHiLXSJ5fNtSaSTeydasfYD1JJ1\n\
PV+yA02XVGYeS/61bWWza1Hrve2aYX6g6LR9tzVGjkF0w0hU7knz4iIIXxjTc0FH\n\
NkapSVAM2m1MKway2V1oKtF1p8wDhRXUkW8uxcARAoGAMgllFVGc4IKdHrGYeXWg\n\
1BuEO6bQ7f8GbC1T7mneLjkDnBrD2tnCfWY5kP3/TT7kkrY1asPwlpSTExmVVyo/\n\
UQXjxbu1JGqfObmc4QrSFW5DhGzylI2gQmY1Vqu+0lJns2f1QbO7/gK9ym0K0oOG\n\
VElzfilssP9NBRaBlYqAStk=\n\
-----END PRIVATE KEY-----";

    const TEST_P12_B64: &str = "\
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";

    #[test]
    fn no_tls_material_returns_none() {
        assert!(client_from_serialized_data("{}").unwrap().is_none());
        assert!(client_from_serialized_data("").unwrap().is_none());
        assert!(client_from_serialized_data(r#"{"api_key":"x"}"#)
            .unwrap()
            .is_none());
    }

    #[test]
    fn pem_cert_and_key_build_client() {
        let json = serde_json::json!({
            "headers": { "Authorization": "Bearer tok" },
            "tls": {
                "client_cert": TEST_CERT,
                "client_key": TEST_KEY,
            }
        });
        assert!(client_from_serialized_data(&json.to_string())
            .unwrap()
            .is_some());
    }

    #[test]
    fn top_level_tls_keys_are_ignored() {
        let json = serde_json::json!({
            "client_cert": TEST_CERT,
            "client_key": TEST_KEY,
        });
        assert!(client_from_serialized_data(&json.to_string())
            .unwrap()
            .is_none());
    }

    #[test]
    fn combined_pem_without_client_key_is_rejected() {
        let combined = format!("{TEST_CERT}\n{TEST_KEY}");
        let json = serde_json::json!({ "tls": { "client_cert": combined } });
        let err = client_from_serialized_data(&json.to_string()).unwrap_err();
        assert!(err.contains("client_key"), "{err}");
    }

    #[test]
    fn file_data_object_is_accepted() {
        let json = serde_json::json!({
            "tls": {
                "client_cert": { "base64": BASE64.encode(TEST_CERT), "filename": "cert.pem" },
                "client_key": { "base64": BASE64.encode(TEST_KEY), "filename": "key.pem" },
            }
        });
        assert!(client_from_serialized_data(&json.to_string())
            .unwrap()
            .is_some());
    }

    #[test]
    fn pkcs12_builds_client() {
        let json = serde_json::json!({
            "tls": {
                "client_p12": TEST_P12_B64,
                "client_p12_password": "testpass",
            }
        });
        assert!(client_from_serialized_data(&json.to_string())
            .unwrap()
            .is_some());
    }

    #[test]
    fn invalid_pem_returns_error() {
        let json = serde_json::json!({
            "tls": {
                "client_cert": "not-a-cert",
                "client_key": TEST_KEY,
            }
        });
        let err = client_from_serialized_data(&json.to_string()).unwrap_err();
        assert!(err.contains("client_cert"), "{err}");
    }

    #[test]
    fn key_without_cert_returns_error() {
        let json = serde_json::json!({ "tls": { "client_key": TEST_KEY } });
        let err = client_from_serialized_data(&json.to_string()).unwrap_err();
        assert!(err.contains("client_key"), "{err}");
    }

    #[test]
    fn extra_ca_without_identity_builds_client() {
        let json = serde_json::json!({ "tls": { "ca_cert": TEST_CERT } });
        assert!(client_from_serialized_data(&json.to_string())
            .unwrap()
            .is_some());
    }

    const TEST_CA2: &str = "-----BEGIN CERTIFICATE-----\n\
MIIDHTCCAgWgAwIBAgIUZLQVyRxhkLAOsgmPt6Uj2cH0sdQwDQYJKoZIhvcNAQEL\n\
BQAwHjEcMBoGA1UEAwwTYXBwLWJyaWRnZS1tdGxzLWNhMjAeFw0yNjEwMTUwNjMx\n\
NTZaFw0zNjEwMTIwNjMxNTZaMB4xHDAaBgNVBAMME2FwcC1icmlkZ2UtbXRscy1j\n\
YTIwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQCqY3OCWzrcwk+GKu1k\n\
NUBgukGk+Ehfssp+VVRNBAEq8PR09W/xEXsOzHcnChGWXk1QWCfSkxy8PTvd+Jyu\n\
PapT9oQeA7w+n1o63fxUmwzEolx0rAvwHGgxmSWnhLHvW13PylJKjpWtfsVPI3hr\n\
Zqt0zmR2bQDLdNFYmxXzfH2uAVQU3/OQwJjW+DaMjukr/dJXhCzJe+EOjgYfOl+j\n\
wd8exnajCtRDTwLtdOd3KHg7cSbHD115X6KR3U1QoDyjTEQI1SxtBfjTfAqK6t3S\n\
dlTy2El5NEPeB9ODVYkeDMJtbnZfzTvr5i8qfuyfJIqoCjz4mLYXBVy5nZ5pWYau\n\
u6bpAgMBAAGjUzBRMB0GA1UdDgQWBBQCjH3KRbnfS2x3tQMuiUi3rknWCTAfBgNV\n\
HSMEGDAWgBQCjH3KRbnfS2x3tQMuiUi3rknWCTAPBgNVHRMBAf8EBTADAQH/MA0G\n\
CSqGSIb3DQEBCwUAA4IBAQBD7uj5NN7i6ned5Yx2yoRwOJN/oUx2d4q5MsQUGHfX\n\
CAY5mWdPUSOpK0H/tlCOjdlgCJ5u6p1HKqlx6lqP/p9PT3pUjZ9lCJWmUS4gYn2h\n\
walfmC8P3H3k/aIOshPLDp0DI7v08F3iNhNN/Tfc8CbbLzNOZvMsC0cp+UK+WGlL\n\
pPaG03Qzrqk8bPtxKYYUM6UNj2Iom4Cwso1jLPkovtl/e+HzquB/C5Kiql4nNL0V\n\
fyDvRF/jWhRxoDmshI1/t5AquEe9nlhtPrVYxzMUphBlUxpD3sWRxGWrirO0yhxE\n\
gxuqrBZ9sE9poJiXVtpNZKwgRspqerGFhp8XQL5DBhcI\n\
-----END CERTIFICATE-----";

    #[test]
    fn ca_cert_pem_bundle_trusts_all_certificates() {
        let bundle = format!("{TEST_CERT}\n{TEST_CA2}");
        let certs = ca_certificates_from_material(bundle.as_bytes()).unwrap();
        assert_eq!(certs.len(), 2);

        let json = serde_json::json!({ "tls": { "ca_cert": bundle } });
        assert!(client_from_serialized_data(&json.to_string())
            .unwrap()
            .is_some());
    }
}
