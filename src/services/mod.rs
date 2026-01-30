use crate::services::autocpu_freq::AutoCpuFreqService;
use crate::services::sddm::SddmService;
use crate::services::sshd::SshdService;
use crate::services::systemd::{SystemdSystemService, SystemdUserService};
use crate::services::tuned::TuneDService;
use crate::services::tuned_ppd::TuneDPPDService;
use crate::services::udev::UdevService;
use crate::{
    services::ssh::{SshSystemService, SshUserService},
    types::managed_service::ManagedService,
};

mod autocpu_freq;
mod sddm;
mod ssh;
mod sshd;
mod systemd;
mod tuned;
mod tuned_ppd;
mod udev;

pub fn get_service_by_name(name: &str) -> Option<Box<dyn ManagedService>> {
    match name {
        "auto-cpufreq" => Some(Box::new(AutoCpuFreqService)),
        "sddm" => Some(Box::new(SddmService)),
        "ssh_user" => Some(Box::new(SshUserService)),
        "ssh_system" => Some(Box::new(SshSystemService)),
        "sshd" => Some(Box::new(SshdService)),
        "systemd_user" => Some(Box::new(SystemdUserService)),
        "systemd_system" => Some(Box::new(SystemdSystemService)),
        "tuned" => Some(Box::new(TuneDService)),
        "tuned-ppd" => Some(Box::new(TuneDPPDService)),
        "udev" => Some(Box::new(UdevService)),
        _ => None,
    }
}
