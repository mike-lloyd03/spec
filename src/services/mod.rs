use crate::services::autocpu_freq::AutoCpuFreqService;
use crate::services::ssh::SshService;
use crate::services::sshd::SshdService;
use crate::services::systemd::SystemdService;
use crate::services::udev::UdevService;

mod autocpu_freq;
mod ssh;
mod sshd;
mod systemd;
pub mod types;
mod udev;

pub fn get_service_by_name(name: &str) -> Option<Box<dyn types::ManagedService>> {
    match name {
        "auto-cpufreq" => Some(Box::new(AutoCpuFreqService)),
        "ssh" => Some(Box::new(SshService)),
        "sshd" => Some(Box::new(SshdService)),
        "systemd" => Some(Box::new(SystemdService)),
        "udev" => Some(Box::new(UdevService)),
        _ => None,
    }
}
