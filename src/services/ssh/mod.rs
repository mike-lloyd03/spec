use crate::adapters::key_value::{BoolStyle, KeyValueAdapter};
use crate::types::managed_service::{
    FileArtifact, ManagedService, ManagedServiceCapability, ServiceState,
};
use crate::types::managed_services_config::ConfigScope;
use crate::types::paths::Paths;

use anyhow::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use toml::Table;

mod enums;
use enums::*;

pub struct SshService;

type SshConfig = IndexMap<String, HostConfig>;

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all(serialize = "PascalCase"))]
struct HostConfig {
    pub host: Option<String>,
    pub add_keys_to_agent: Option<AddKeysToAgent>,
    pub address_family: Option<AddressFamily>,
    pub batch_mode: Option<bool>,
    pub bind_address: Option<String>,
    pub bind_interface: Option<String>,
    pub canonical_domains: Option<String>,
    pub canonicalize_fallback_local: Option<bool>,
    pub canonicalize_hostname: Option<CanonicalizeHostname>,
    pub canonicalize_max_dots: Option<u16>,
    pub canonicalize_permitted_cnames: Option<String>,
    pub ca_signature_algorithms: Option<String>,
    pub certificate_file: Option<Vec<String>>,
    pub channel_timeout: Option<String>,
    pub check_host_ip: Option<bool>,
    pub ciphers: Option<String>,
    pub clear_all_forwardings: Option<bool>,
    pub compression: Option<bool>,
    pub connection_attempts: Option<u16>,
    pub connect_timeout: Option<u16>,
    pub control_master: Option<YesNoAskAuto>,
    pub control_path: Option<String>,
    pub control_persist: Option<ControlPersist>,
    pub dynamic_forward: Option<Vec<String>>,
    pub enable_escape_commandline: Option<String>,
    pub enable_ssh_keysign: Option<bool>,
    pub escape_char: Option<String>,
    pub exit_on_forward_failure: Option<bool>,
    pub fingerprint_hash: Option<String>,
    pub fork_after_authentication: Option<bool>,
    pub forward_agent: Option<YesNoPath>,
    pub forward_x11: Option<bool>,
    pub forward_x11_timeout: Option<String>,
    pub forward_x11_trusted: Option<bool>,
    pub gateway_ports: Option<bool>,
    pub global_known_hosts_file: Option<Vec<String>>,
    pub gssapi_authentication: Option<bool>,
    pub gssapi_delegate_credentials: Option<bool>,
    pub hash_known_hosts: Option<bool>,
    pub hostbased_accepted_algorithms: Option<String>,
    pub hostbased_authentication: Option<bool>,
    pub host_key_algorithms: Option<String>,
    pub host_key_alias: Option<String>,
    pub hostname: Option<String>,
    pub identities_only: Option<bool>,
    pub identity_agent: Option<String>,
    pub identity_file: Option<String>,
    pub ignore_unknown: Option<String>,
    pub include: Option<Vec<String>>,
    pub ip_qos: Option<String>,
    pub kbd_interactive_authentication: Option<bool>,
    pub kbd_interactive_devices: Option<String>,
    pub kex_algorithms: Option<String>,
    pub known_host_command: Option<String>,
    pub local_command: Option<String>,
    pub local_forward: Option<Vec<String>>, // "8080 localhost:80"
    pub log_level: Option<LogLevel>,
    pub log_verbose: Option<String>,
    pub macs: Option<String>,
    pub no_host_authentication_for_localhost: Option<bool>,
    pub number_of_password_prompts: Option<u16>,
    pub obscure_keystroke_timing: Option<u16>,
    pub password_authentication: Option<bool>,
    pub permit_local_command: Option<bool>,
    pub permit_remote_open: Option<Vec<String>>,
    pub pkcs11_provider: Option<String>,
    pub port: Option<u16>,
    pub preferred_authentications: Option<String>,
    pub proxy_command: Option<String>,
    pub proxy_jump: Option<String>,
    pub proxy_use_fdpass: Option<bool>,
    pub pubkey_accepted_algorithms: Option<String>,
    pub pubkey_authentication: Option<PubkeyAuthentication>,
    pub refuse_connection: Option<bool>,
    pub rekey_limit: Option<String>,
    pub remote_command: Option<String>,
    pub remote_forward: Option<Vec<String>>,
    pub request_tty: Option<RequestTty>,
    pub required_rsasize: Option<u16>,
    pub revoked_host_keys: Option<String>,
    pub security_key_provider: Option<String>,
    pub send_env: Option<Vec<String>>,
    pub server_alive_count_max: Option<u16>,
    pub server_alive_interval: Option<u16>,
    pub session_type: Option<String>,
    pub set_env: Option<Vec<String>>,
    pub stdin_null: Option<bool>,
    pub stream_local_bind_mask: Option<u16>,
    pub stream_local_bind_unlink: Option<bool>,
    pub strict_host_key_checking: Option<StrictHostKeyChecking>,
    pub syslog_facility: Option<String>,
    pub tcp_keep_alive: Option<bool>,
    pub tag: Option<String>,
    pub tunnel: Option<Tunnel>,
    pub tunnel_device: Option<String>,
    pub update_host_keys: Option<YesNoAsk>,
    pub user: Option<String>,
    pub user_known_hosts_file: Option<Vec<String>>,
    pub verify_host_key_dns: Option<YesNoAsk>,
    pub version_addendum: Option<String>,
    pub visual_host_key: Option<bool>,
    pub warn_weak_crypto: Option<bool>,
    pub xauth_location: Option<String>,
}

impl ManagedService for SshService {
    fn name(&self) -> &str {
        "ssh"
    }

    fn capabilities(&self) -> ManagedServiceCapability {
        ManagedServiceCapability::UserAndSystem
    }

    fn plan(
        &self,
        config_table: &Table,
        config_scope: ConfigScope,
        paths: &Paths,
    ) -> Result<(Vec<FileArtifact>, Option<ServiceState>)> {
        let config: SshConfig = self.parse_config(config_table)?;

        let mut adapter = KeyValueAdapter::new(" ", "#").bool_style(BoolStyle::YesNo);

        adapter.comment("Managed by spec");

        let path = match config_scope {
            ConfigScope::User => paths.user_config.join("ssh/config"),
            ConfigScope::System => paths.system_config.join("ssh/ssh_config"),
        };

        for (k, v) in config {
            adapter
                .set("Host", k)
                .indent()
                .parse_struct(&v)?
                .outdent()
                .empty_line();
        }

        let file = FileArtifact {
            path,
            content: adapter.build(),
            permissions: 0o644,
        };

        Ok((vec![file], None))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_plan() -> Result<()> {
        let config_table = Table::from_str(
            r#"
[host1]
hostname = "192.168.1.100"
user = "user1"
identity_file = "~/.ssh/id_rsa"

[host2]
hostname = "192.168.1.101"
user = "user2"
identity_file = "~/.ssh/custom_key"
        "#,
        )?;

        let ssh_service = SshService;
        let paths = Paths::default();

        let (files, _) = ssh_service.plan(&config_table, ConfigScope::System, &paths)?;

        assert_eq!(files.len(), 1);

        if let Some(file) = files.first() {
            assert_eq!(
                file.path.to_string_lossy().to_string(),
                "ssh/ssh_config".to_string()
            );

            let expected_content = r#"# Managed by spec
Host host1
    Hostname 192.168.1.100
    IdentityFile ~/.ssh/id_rsa
    User user1

Host host2
    Hostname 192.168.1.101
    IdentityFile ~/.ssh/custom_key
    User user2
"#;

            assert_eq!(expected_content, file.content);
        }

        Ok(())
    }
}
