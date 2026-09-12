// https://github.com/opencontainers/image-spec/blob/main/config.md

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::specs::descriptor::Digest;

/// Defines the execution parameters for use within a container runtime.
/// Its corresponding media type is `application/vnd.oci.image.config.v1+json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageConfiguration {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub architecture: String,
    pub os: String,
    // The JSON keys "os.version" and "os.features" contain literal dots,
    // so rename_all = "camelCase" would produce the wrong names ("osVersion").
    #[serde(
        default,
        rename = "os.version",
        skip_serializing_if = "Option::is_none"
    )]
    pub os_version: Option<String>,
    #[serde(
        default,
        rename = "os.features",
        skip_serializing_if = "Option::is_none"
    )]
    pub os_features: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<Config>,
    pub rootfs: RootFs,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<History>>,
}

/// A set of strings serialized as a JSON object mapping each key to an
/// empty object, e.g. `{"8080/tcp": {}, "9090/udp": {}}`.
///
/// This unusual shape is a direct JSON serialization of the Go type
/// `map[string]struct{}`, which the spec inherits from Docker for
/// `ExposedPorts` and `Volumes`. Only the keys carry meaning.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StringSet(pub HashSet<String>);

/// Accepts only an empty JSON object `{}`.
/// `deny_unknown_fields` rejects any value with content (e.g. `{"junk": 1}`)
/// at parse time, enforcing the spec's requirement that these values are
/// always empty.
///
/// Note: this must be `struct EmptyObject {}` (an empty struct), not
/// `struct EmptyObject;` (a unit struct). Serde serializes a unit struct
/// as `null`, but an empty struct as `{}`.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyObject {}

impl Serialize for StringSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Delegate to HashMap's serializer: each key maps to EmptyObject,
        // which serializes as {}.
        let mut map = HashMap::new();
        for key in &self.0 {
            map.insert(key, EmptyObject {});
        }
        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StringSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Delegate to HashMap's deserializer: EmptyObject validates that
        // every value is {}, then we keep only the keys.
        let map = HashMap::<String, EmptyObject>::deserialize(deserializer)?;
        let mut set = HashSet::new();
        for key in map.keys() {
            set.insert(key.clone());
        }
        Ok(StringSet(set))
    }
}

/// Execution parameters which should be used as a base when running a
/// container.
///
/// Note: field names are PascalCase in JSON (`"User"`, `"Cmd"`), a Docker
/// legacy convention. This is the only struct in the spec that does this.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Keys are `port/tcp`, `port/udp`, or `port` (default protocol: tcp).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exposed_ports: Option<StringSet>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cmd: Option<Vec<String>>,
    /// Directories likely to hold container-instance-specific data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volumes: Option<StringSet>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_signal: Option<String>,
}

/// References the layer content addresses used by the image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootFs {
    #[serde(rename = "type", deserialize_with = "deserialize_rootfs_type")]
    pub fs_type: String,
    /// Digests of each layer's uncompressed tar archive, ordered first to
    /// last. Not to be confused with the layer digests in the manifest,
    /// which are computed over the compressed blobs.
    pub diff_ids: Vec<Digest>,
}

// The spec requires rootfs.type to be exactly "layers". Same validation
// pattern as deserialize_schema_version in manifest.rs.
fn deserialize_rootfs_type<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(d)?;
    if value != "layers" {
        return Err(serde::de::Error::custom(format!(
            r#"rootfs type must be "layers", got: {value}"#
        )));
    }
    Ok(value)
}

/// Describes the history of a single layer. Purely informational.
///
/// Entries with `empty_layer: true` correspond to build instructions that
/// produced no filesystem change (e.g. ENV, CMD), so `history` may contain
/// more entries than `rootfs.diff_ids`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct History {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty_layer: Option<bool>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rstest::rstest;
    use serde_json::json;

    use super::*;

    const DIFF_ID_A: &str =
        "sha256:c6f988f4874bb0add23a778f753c65efe992244e148a1d2ec2a8b664fb66bbd1";
    const DIFF_ID_B: &str =
        "sha256:5f70bf18a086007016e948b04aed3b82103a36bea41755b6cddfaf10ace3c6ef";

    impl Default for ImageConfiguration {
        fn default() -> Self {
            Self {
                created: None,
                author: None,
                architecture: "amd64".to_owned(),
                os: "linux".to_owned(),
                os_version: None,
                os_features: None,
                variant: None,
                config: None,
                rootfs: RootFs {
                    fs_type: "layers".to_owned(),
                    diff_ids: vec![Digest::try_from(DIFF_ID_A.to_string()).unwrap()],
                },
                history: None,
            }
        }
    }

    fn valid_rootfs_json() -> serde_json::Value {
        json!({
            "type": "layers",
            "diff_ids": [DIFF_ID_A, DIFF_ID_B]
        })
    }

    // --- StringSet ---

    #[test]
    fn string_set_serializes_as_map_with_empty_object_values() {
        let mut set = HashSet::new();
        set.insert("8080/tcp".to_owned());
        set.insert("9090/udp".to_owned());

        let string_set = StringSet(set);
        let serialized = serde_json::to_value(&string_set).unwrap();

        assert_eq!(serialized["8080/tcp"], json!({}));
        assert_eq!(serialized["9090/udp"], json!({}));
    }

    #[test]
    fn string_set_deserializes_keys_from_map_with_empty_object_values() {
        let json = json!({
            "8080/tcp": {},
            "9090/udp": {}
        });

        let string_set = serde_json::from_value::<StringSet>(json).unwrap();

        assert!(string_set.0.contains("8080/tcp"));
        assert!(string_set.0.contains("9090/udp"));
        assert_eq!(string_set.0.len(), 2);
    }

    #[test]
    fn string_set_rejects_non_empty_object_values() {
        let json = json!({
            "8080/tcp": {"junk": "this should not be here"}
        });

        let err = serde_json::from_value::<StringSet>(json).unwrap_err();

        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn string_set_round_trips_correctly() {
        let json = json!({
            "8080/tcp": {}
        });

        let string_set = serde_json::from_value::<StringSet>(json.clone()).unwrap();
        let serialized = serde_json::to_value(&string_set).unwrap();

        assert_eq!(serialized, json);
    }

    #[test]
    fn string_set_empty_set_serializes_as_empty_object() {
        let string_set = StringSet(HashSet::new());
        let serialized = serde_json::to_value(&string_set).unwrap();

        assert_eq!(serialized, json!({}));
    }

    // --- ImageConfiguration ---

    #[test]
    fn deserialize_accepts_full_spec_example() {
        let json = json!({
            "created": "2015-10-31T22:22:56.015925234Z",
            "author": "Alyssa P. Hacker <alyspdev@example.com>",
            "architecture": "amd64",
            "os": "linux",
            "config": {
                "User": "alice",
                "ExposedPorts": { "8080/tcp": {} },
                "Env": ["PATH=/usr/local/sbin:/usr/local/bin", "FOO=oci_is_a"],
                "Entrypoint": ["/bin/my-app-binary"],
                "Cmd": ["--foreground", "--config", "/etc/my-app.d/default.cfg"],
                "Volumes": { "/var/job-result-data": {} },
                "WorkingDir": "/home/alice",
                "Labels": { "com.example.project.git.url": "https://example.com/project.git" }
            },
            "rootfs": valid_rootfs_json(),
            "history": [
                { "created": "2015-10-31T22:22:54Z", "created_by": "/bin/sh -c #(nop) ADD file:abc in /" },
                { "created": "2015-10-31T22:22:55Z", "created_by": "/bin/sh -c #(nop) CMD [\"sh\"]", "empty_layer": true }
            ]
        });

        let config = serde_json::from_value::<ImageConfiguration>(json).unwrap();

        assert_eq!(config.architecture, "amd64");
        assert_eq!(config.os, "linux");
        assert_eq!(config.rootfs.diff_ids.len(), 2);
        assert!(config.config.is_some());
        assert_eq!(config.history.unwrap().len(), 2);
    }

    #[test]
    fn deserialize_accepts_minimal_config() {
        let json = json!({
            "architecture": "amd64",
            "os": "linux",
            "rootfs": valid_rootfs_json()
        });

        let config = serde_json::from_value::<ImageConfiguration>(json).unwrap();

        assert!(config.config.is_none());
        assert!(config.history.is_none());
        assert!(config.author.is_none());
    }

    #[rstest]
    #[case::missing_architecture(json!({ "os": "linux", "rootfs": valid_rootfs_json() }))]
    #[case::missing_os(json!({ "architecture": "amd64", "rootfs": valid_rootfs_json() }))]
    #[case::missing_rootfs(json!({ "architecture": "amd64", "os": "linux" }))]
    fn deserialize_rejects_missing_required_fields(#[case] json: serde_json::Value) {
        assert!(serde_json::from_value::<ImageConfiguration>(json).is_err());
    }

    #[test]
    fn deserialize_accepts_os_version_and_features() {
        let json = json!({
            "architecture": "amd64",
            "os": "windows",
            "os.version": "10.0.14393.1066",
            "os.features": ["win32k"],
            "rootfs": valid_rootfs_json()
        });

        let config = serde_json::from_value::<ImageConfiguration>(json).unwrap();

        assert_eq!(config.os_version.unwrap(), "10.0.14393.1066");
        assert_eq!(config.os_features.unwrap(), vec!["win32k"]);
    }

    #[test]
    fn serializes_os_version_with_dot_notation_not_camel_case() {
        let config = ImageConfiguration {
            os_version: Some("10.0.14393.1066".to_owned()),
            ..Default::default()
        };

        let serialized = serde_json::to_value(&config).unwrap();

        assert!(serialized.get("os.version").is_some());
        assert!(serialized.get("osVersion").is_none());
    }

    // --- RootFs ---

    #[test]
    fn deserialize_rejects_non_layers_rootfs_type() {
        let json = json!({
            "architecture": "amd64",
            "os": "linux",
            "rootfs": {
                "type": "overlay",
                "diff_ids": [DIFF_ID_A]
            }
        });

        let err = serde_json::from_value::<ImageConfiguration>(json).unwrap_err();

        assert!(err.to_string().contains(r#"rootfs type must be "layers""#));
    }

    #[test]
    fn rootfs_diff_ids_parse_as_typed_digests() {
        let rootfs = serde_json::from_value::<RootFs>(valid_rootfs_json()).unwrap();

        assert_eq!(rootfs.diff_ids[0], Digest::try_from(DIFF_ID_A.to_string()).unwrap());
        assert_eq!(rootfs.diff_ids[1], Digest::try_from(DIFF_ID_B.to_string()).unwrap());
    }

    // --- Config ---

    #[test]
    fn config_serializes_fields_as_pascal_case() {
        let config = Config {
            user: Some("alice".to_owned()),
            working_dir: Some("/home/alice".to_owned()),
            stop_signal: Some("SIGKILL".to_owned()),
            ..Default::default()
        };

        let serialized = serde_json::to_value(&config).unwrap();

        assert!(serialized.get("User").is_some());
        assert!(serialized.get("WorkingDir").is_some());
        assert!(serialized.get("StopSignal").is_some());
        assert!(serialized.get("user").is_none());
        assert!(serialized.get("working_dir").is_none());
    }

    #[test]
    fn config_deserializes_pascal_case_fields() {
        let json = json!({
            "Entrypoint": ["/bin/app"],
            "Cmd": ["--flag"],
            "Env": ["FOO=bar"],
            "ExposedPorts": { "8080/tcp": {} }
        });

        let config = serde_json::from_value::<Config>(json).unwrap();

        assert_eq!(config.entrypoint.unwrap(), vec!["/bin/app"]);
        assert_eq!(config.cmd.unwrap(), vec!["--flag"]);
        assert_eq!(config.env.unwrap(), vec!["FOO=bar"]);
        assert!(config.exposed_ports.is_some());
    }

    #[test]
    fn config_deserializes_exposed_ports_and_volumes_as_sets() {
        let json = json!({
            "ExposedPorts": {
                "8080/tcp": {},
                "9090/udp": {}
            },
            "Volumes": {
                "/var/log/kehai": {},
                "/var/data": {}
            }
        });

        let config = serde_json::from_value::<Config>(json).unwrap();

        let ports = config.exposed_ports.unwrap();
        assert!(ports.0.contains("8080/tcp"));
        assert!(ports.0.contains("9090/udp"));
        assert_eq!(ports.0.len(), 2);

        let volumes = config.volumes.unwrap();
        assert!(volumes.0.contains("/var/log/kehai"));
        assert!(volumes.0.contains("/var/data"));
        assert_eq!(volumes.0.len(), 2);
    }

    #[test]
    fn config_rejects_exposed_ports_with_non_empty_values() {
        let json = json!({
            "ExposedPorts": {
                "8080/tcp": {"should_not": "be_here"}
            }
        });

        let err = serde_json::from_value::<Config>(json).unwrap_err();

        assert!(err.to_string().contains("unknown field"));
    }

    // --- History ---

    #[test]
    fn history_empty_layer_true_marks_metadata_only_layers() {
        let json = json!({
            "created_by": "/bin/sh -c #(nop) CMD [\"sh\"]",
            "empty_layer": true
        });

        let history = serde_json::from_value::<History>(json).unwrap();

        assert_eq!(history.empty_layer, Some(true));
    }

    #[test]
    fn history_serialization_skips_none_optional_fields() {
        let history = History {
            created_by: Some("/bin/sh -c apk add curl".to_owned()),
            ..Default::default()
        };

        let serialized = serde_json::to_value(&history).unwrap();

        assert_eq!(
            serialized,
            json!({ "created_by": "/bin/sh -c apk add curl" })
        );
    }
}