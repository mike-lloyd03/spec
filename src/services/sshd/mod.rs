use crate::adapters::key_value::{BoolStyle, KeyValueAdapter};
use crate::services::{FileArtifact, ManagedService, ServiceConfig, ServiceState};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use toml::Table;
mod enums;
use enums::*;

pub struct SshdService;

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all(serialize = "PascalCase"))]
struct SshdConfig {
    pub accept_env: Option<Vec<String>>,
    pub address_family: Option<AddressFamily>,
    pub allow_agent_forwarding: Option<bool>,
    pub allow_groups: Option<Vec<String>>,
    pub allow_stream_local_forwarding: Option<AllowForwarding>,
    pub allow_tcp_forwarding: Option<AllowForwarding>,
    pub allow_users: Option<Vec<String>>,
    pub authentication_methods: Option<Vec<String>>,
    pub authorized_keys_command: Option<String>,
    pub authorized_keys_command_user: Option<String>,
    pub authorized_keys_file: Option<Vec<String>>,
    pub authorized_principals_command: Option<String>,
    pub authorized_principals_command_user: Option<String>,
    pub authorized_principals_file: Option<String>,
    pub banner: Option<String>,
    pub ca_signature_algorithms: Option<String>,
    pub channel_timeout: Option<Vec<String>>,
    pub chroot_directory: Option<String>,
    pub ciphers: Option<String>,
    pub client_alive_count_max: Option<u32>,
    pub client_alive_interval: Option<u32>,
    pub compression: Option<Compression>,
    pub deny_groups: Option<Vec<String>>,
    pub deny_users: Option<Vec<String>>,
    pub disable_forwarding: Option<bool>,
    pub expose_auth_info: Option<bool>,
    pub fingerprint_hash: Option<FingerprintHash>,
    pub force_command: Option<String>,
    pub gateway_ports: Option<GatewayPorts>,
    pub gssapi_authentication: Option<bool>,
    pub gssapi_cleanup_credentials: Option<bool>,
    pub gssapi_strict_acceptor_check: Option<bool>,
    pub hostbased_accepted_algorithms: Option<String>,
    pub hostbased_authentication: Option<bool>,
    pub hostbased_uses_name_from_packet_only: Option<bool>,
    pub host_certificate: Option<String>,
    pub host_key: Option<Vec<String>>,
    pub host_key_agent: Option<String>,
    pub host_key_algorithms: Option<String>,
    pub ignore_rhosts: Option<IgnoreRhosts>,
    pub ignore_user_known_hosts: Option<bool>,
    pub ip_qos: Option<String>,
    pub kbd_interactive_authentication: Option<bool>,
    pub kerberos_authentication: Option<bool>,
    pub kerberos_get_afs_token: Option<bool>,
    pub kerberos_or_local_passwd: Option<bool>,
    pub kerberos_ticket_cleanup: Option<bool>,
    pub kex_algorithms: Option<String>,
    pub listen_address: Option<Vec<String>>,
    pub login_grace_time: Option<String>,
    pub log_level: Option<LogLevel>,
    pub log_verbose: Option<Vec<String>>,
    pub macs: Option<String>,

    // "Match" is a conditional block.
    // Usually handled via a custom serializer or separate structs.
    #[serde(rename = "Match")]
    pub match_blocks: Option<Vec<String>>,

    pub max_auth_tries: Option<u32>,
    pub max_sessions: Option<u32>,
    pub max_startups: Option<String>,
    pub moduli_file: Option<String>,
    pub pam_service_name: Option<String>,
    pub password_authentication: Option<bool>,
    pub permit_empty_passwords: Option<bool>,
    pub permit_listen: Option<Vec<String>>,
    pub permit_open: Option<Vec<String>>,
    pub permit_root_login: Option<PermitRootLogin>,
    pub permit_tty: Option<bool>,
    pub permit_tunnel: Option<PermitTunnel>,
    pub permit_user_environment: Option<String>,
    pub permit_user_rc: Option<bool>,
    pub per_source_max_startups: Option<String>,
    pub per_source_net_block_size: Option<String>,
    pub per_source_penalties: Option<String>,
    pub per_source_penalty_exempt_list: Option<String>,
    pub pid_file: Option<String>,
    pub port: Option<Vec<u16>>,
    pub print_last_log: Option<bool>,
    pub print_motd: Option<bool>,
    pub pubkey_accepted_algorithms: Option<String>,
    pub pubkey_auth_options: Option<String>,
    pub pubkey_authentication: Option<bool>,
    pub refuse_connection: Option<bool>,
    pub rekey_limit: Option<String>,
    pub required_rsa_size: Option<u32>,
    pub revoked_keys: Option<String>,
    pub r_domain: Option<String>,
    pub security_key_provider: Option<String>,
    pub set_env: Option<Vec<String>>,
    pub sshd_auth_path: Option<String>,
    pub sshd_session_path: Option<String>,
    pub stream_local_bind_mask: Option<String>,
    pub stream_local_bind_unlink: Option<bool>,
    pub strict_modes: Option<bool>,
    pub subsystem: Option<Vec<String>>,
    pub syslog_facility: Option<SyslogFacility>,
    pub tcp_keep_alive: Option<bool>,
    pub trusted_user_ca_keys: Option<String>,
    pub unused_connection_timeout: Option<String>,
    pub use_dns: Option<bool>,
    pub use_pam: Option<bool>,
    pub version_addendum: Option<String>,
    pub x11_display_offset: Option<u32>,
    pub x11_forwarding: Option<bool>,
    pub x11_use_localhost: Option<bool>,
    pub x_auth_location: Option<String>,

    #[serde(skip_serializing)]
    pub service: Option<ServiceConfig>,
}

impl ManagedService for SshdService {
    fn name(&self) -> &str {
        "sshd"
    }

    fn plan(&self, config_table: &Table) -> Result<(Vec<FileArtifact>, Option<ServiceState>)> {
        let config: SshdConfig = self.parse_config(config_table)?;

        let mut adapter = KeyValueAdapter::new(" ", "#").bool_style(BoolStyle::YesNo);

        adapter.comment("Managed by tenant");

        adapter.parse_struct(&config)?;

        let file = FileArtifact {
            path: PathBuf::from("/etc/ssh/sshd_config"),
            content: adapter.build(),
            permissions: 0o644,
        };

        let service_state = if let Some(state) = config.service {
            ServiceState {
                name: self.name().to_string(),
                enabled: state.enabled.unwrap_or_default(),
                running: state.running.unwrap_or_default(),
            }
        } else {
            ServiceState::default()
        };

        Ok((vec![file], Some(service_state)))
    }
}
