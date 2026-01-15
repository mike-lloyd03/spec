// use crate::services::autocpu_freq::AutoCpuFreqService;
use crate::services::sshd::SshdService;
use crate::services::systemd::SystemdService;
use crate::services::udev::UdevService;
use crate::{
    services::ssh::{SshSystemService, SshUserService},
    types::managed_service::ManagedService,
};

// mod autocpu_freq;
mod ssh;
mod sshd;
mod systemd;
mod udev;

pub fn get_service_by_name(name: &str) -> Option<Box<dyn ManagedService>> {
    match name {
        // "auto-cpufreq" => Some(Box::new(AutoCpuFreqService)),
        "ssh_user" => Some(Box::new(SshUserService)),
        "ssh_system" => Some(Box::new(SshSystemService)),
        "sshd" => Some(Box::new(SshdService)),
        "systemd" => Some(Box::new(SystemdService)),
        "udev" => Some(Box::new(UdevService)),
        _ => None,
    }
}
