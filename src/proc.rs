use serde::Serialize;
use std::fs;

#[derive(Serialize, Debug, Clone)]
pub struct Process { // [1]
    pub pid:     u32,
    pub name:    String,
    pub state:   String,
    pub ppid:    u32,
    pub vmrss:   u64,
    pub cmdline: String,
    pub user:    String,
}

pub fn read_processes() -> Vec<Process> { // [2]
    let mut procs = Vec::new();

    let Ok(entries) = fs::read_dir("/proc") else { // [3]
        return procs;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        let Ok(pid) = name_str.parse::<u32>() else { // [4]
            continue;
        };

        let proc_path = format!("/proc/{}", pid);

        let status_path = format!("{}/status", proc_path);
        let Ok(status_raw) = fs::read_to_string(&status_path) else { // [5]
            continue;
        };

        let mut proc_name = String::new();
        let mut state     = String::new();
        let mut ppid      = 0u32;
        let mut vmrss     = 0u64;
        let mut uid       = 0u32;

        for line in status_raw.lines() {
            if let Some(val) = line.strip_prefix("Name:\t") {
                proc_name = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("State:\t") {
                state = val.chars().next().unwrap_or('?').to_string(); // [6]
            } else if let Some(val) = line.strip_prefix("PPid:\t") {
                ppid = val.trim().parse().unwrap_or(0);
            } else if let Some(val) = line.strip_prefix("VmRSS:\t") {
                vmrss = val.split_whitespace().next()
                           .and_then(|v| v.parse().ok())
                           .unwrap_or(0); // [7]
            } else if let Some(val) = line.strip_prefix("Uid:\t") {
                uid = val.split_whitespace().next()
                          .and_then(|v| v.parse().ok())
                          .unwrap_or(0); // [8]
            }
        }

        let cmdline = fs::read(format!("{}/cmdline", proc_path))
            .map(|bytes| {
                bytes.iter()
                     .map(|&b| if b == 0 { ' ' } else { b as char }) // [9]
                     .collect::<String>()
                     .trim()
                     .to_string()
            })
            .unwrap_or_default();

        let user = match uid { // [10]
            0     => "root".to_string(),
            65534 => "nobody".to_string(),
            _     => format!("{}", uid),
        };

        procs.push(Process {
            pid,
            name: proc_name.clone(),
            state: state_label(&state),
            ppid,
            vmrss,
            cmdline: if cmdline.is_empty() {
                format!("[{}]", proc_name) // [11]
            } else {
                cmdline
            },
            user,
        });
    }

    procs.sort_by_key(|p| p.pid); // [12]
    procs
}

fn state_label(state: &str) -> String { // [13]
    match state {
        "R" => "Running".to_string(),
        "S" => "Sleeping".to_string(),
        "D" => "Waiting".to_string(),
        "Z" => "Zombie".to_string(),
        "T" => "Stopped".to_string(),
        "I" => "Idle".to_string(),
        _   => state.to_string(), // [14]
    }
}

pub fn kill_process(pid: u32) -> Result<(), String> { // [15]
    if pid <= 1 {
        return Err(format!("Refusing to kill PID {} (system process)", pid)); // [16]
    }

    let status = std::process::Command::new("kill")
        .arg(pid.to_string())
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("kill {} failed — process may have already exited", pid)) // [17]
    }
}
