use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum AddressFamily {
    Any,
    Inet,
    Inet6,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum AllowForwarding {
    Yes,
    All,
    No,
    Local,
    Remote,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Compression {
    Yes,
    Delayed,
    No,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum FingerprintHash {
    Md5,
    Sha256,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum GatewayPorts {
    Yes,
    No,
    Clientspecified,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum IgnoreRhosts {
    Yes,
    ShostsOnly,
    No,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Quiet,
    Fatal,
    Error,
    Info,
    Verbose,
    Debug,
    Debug1,
    Debug2,
    Debug3,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum PermitRootLogin {
    Yes,
    ProhibitPassword,
    ForcedCommandsOnly,
    No,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum PermitTunnel {
    Yes,
    PointToPoint,
    Ethernet,
    No,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum SyslogFacility {
    Daemon,
    User,
    Auth,
    Local0,
    Local1,
    Local2,
    Local3,
    Local4,
    Local5,
    Local6,
    Local7,
}
