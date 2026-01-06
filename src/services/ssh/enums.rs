use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum AddKeysToAgent {
    Yes,
    No,
    Ask,
    Confirm,
    #[serde(untagged)]
    Time(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum StrictHostKeyChecking {
    Yes,
    No,
    Ask,
    AcceptNew,
    Off,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum AddressFamily {
    Any,
    Inet,
    Inet6,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum CanonicalizeHostname {
    Yes,
    No,
    Always,
    None,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum RequestTty {
    Yes,
    No,
    Force,
    Auto,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Tunnel {
    Yes,
    PointToPoint,
    Ethernet,
    No,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum PubkeyAuthentication {
    Yes,
    No,
    Unbound,
    HostBound,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum YesNoAsk {
    Yes,
    No,
    Ask,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum YesNoAskAuto {
    Yes,
    No,
    Ask,
    Auto,
    AutoAsk,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum YesNoPath {
    Yes,
    No,
    Path,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ControlPersist {
    Yes,
    No,
    #[serde(untagged)]
    Time(String),
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Quiet,
    Fatal,
    Error,
    #[default]
    Info,
    Verbose,
    Debug,
    Debug1,
    Debug2,
    Debug3,
}
