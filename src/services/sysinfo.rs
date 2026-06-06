use sysinfo::{Disks, Networks, System};

#[derive(Debug, Clone)]
pub struct SysInfoSnapshot {
    pub hostname: String,
    pub kernel: String,
    pub os: String,
    pub uptime: u64,
    pub cpu_brand: String,
    pub cpu_count: usize,
    pub cpu_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub disk_info: Vec<DiskInfo>,
    pub network_info: Vec<NetworkInfo>,
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub mount: String,
    pub total: u64,
    pub used: u64,
}

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
}

pub fn gather_snapshot() -> SysInfoSnapshot {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpus = sys.cpus();
    let cpu_usage = cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32;

    let disks = Disks::new_with_refreshed_list();
    let disk_info = disks
        .iter()
        .map(|d| DiskInfo {
            mount: d.mount_point().to_string_lossy().into_owned(),
            total: d.total_space(),
            used: d.total_space() - d.available_space(),
        })
        .collect();

    let networks = Networks::new_with_refreshed_list();
    let network_info = networks
        .iter()
        .map(|(name, data)| NetworkInfo {
            name: name.clone(),
            received: data.total_received(),
            transmitted: data.total_transmitted(),
        })
        .collect();

    SysInfoSnapshot {
        hostname: System::host_name().unwrap_or_default(),
        kernel: System::kernel_version().unwrap_or_default(),
        os: System::long_os_version().unwrap_or_default(),
        uptime: System::uptime(),
        cpu_brand: cpus
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_default(),
        cpu_count: cpus.len(),
        cpu_usage,
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        total_swap: sys.total_swap(),
        used_swap: sys.used_swap(),
        disk_info,
        network_info,
    }
}
