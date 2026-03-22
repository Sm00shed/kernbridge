use anyhow::Result;
use askama::Template;

#[derive(Template)]
#[template(path = "sqm/index.html")]
pub struct SqmTemplate {
    pub config: SqmConfig,
}

#[allow(non_snake_case)]
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SqmConfig {
    pub enabled: String,
    pub download: String,
    pub upload: String,
    pub debug_logging: String,
    pub verbosity: String,
    pub qdisc: String,
    pub script: String,
    pub qdisc_advanced: String,
    pub use_mq: String,
    pub squash_dscp: String,
    pub squash_ingress: String,
    pub ingress_ecn: String,
    pub egress_ecn: String,
    pub qdisc_really_really_advanced: String,
    pub ilimit: String,
    pub elimit: String,
    pub itarget: String,
    pub etarget: String,
    pub iqdisc_opts: String,
    pub eqdisc_opts: String,
    pub linklayer: String,
    pub overhead: String,
    pub linklayer_advanced: String,
    pub tcMTU: String,
    pub tcTSIZE: String,
    pub tcMPU: String,
    pub linklayer_adaptation_mechanism: String,
}

#[allow(non_snake_case)]
#[derive(serde::Deserialize, Debug)]
pub struct SqmForm {
    pub enabled: Option<String>,
    pub download: String,
    pub upload: String,
    pub debug_logging: Option<String>,
    pub verbosity: String,
    pub qdisc: String,
    pub script: String,
    pub qdisc_advanced: Option<String>,
    pub use_mq: Option<String>,
    pub squash_dscp: String,
    pub squash_ingress: String,
    pub ingress_ecn: String,
    pub egress_ecn: String,
    pub qdisc_really_really_advanced: Option<String>,
    pub ilimit: String,
    pub elimit: String,
    pub itarget: String,
    pub etarget: String,
    pub iqdisc_opts: String,
    pub eqdisc_opts: String,
    pub linklayer: String,
    pub overhead: String,
    pub linklayer_advanced: Option<String>,
    pub tcMTU: String,
    pub tcTSIZE: String,
    pub tcMPU: String,
    pub linklayer_adaptation_mechanism: String,
}

fn get_sqm_section() -> String {
    std::process::Command::new("uci")
        .args(["show", "sqm"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .find(|l| l.contains("=queue"))
                .and_then(|l| l.split('.').nth(1))
                .and_then(|l| l.split('=').next())
                .unwrap_or("@sqm[0]")
                .to_string()
        })
        .unwrap_or_else(|_| "@sqm[0]".to_string())
}

pub fn read_sqm_config() -> SqmConfig {
    let section = get_sqm_section();
    SqmConfig {
        enabled: uci_get(&format!("sqm.{}.enabled", section), ""),
        download: uci_get(&format!("sqm.{}.download", section), ""),
        upload: uci_get(&format!("sqm.{}.upload", section), ""),
        debug_logging: uci_get(&format!("sqm.{}.debug_logging", section), ""),
        verbosity: uci_get(&format!("sqm.{}.verbosity", section), "5"),
        qdisc: uci_get(&format!("sqm.{}.qdisc", section), "cake"),
        script: uci_get(&format!("sqm.{}.script", section), "piece_of_cake.qos"),
        qdisc_advanced: uci_get(&format!("sqm.{}.qdisc_advanced", section), "false"),
        use_mq: uci_get(&format!("sqm.{}.use_mq", section), "false"),
        squash_dscp: uci_get(&format!("sqm.{}.squash_dscp", section), "1"),
        squash_ingress: uci_get(&format!("sqm.{}.squash_ingress", section), "1"),
        ingress_ecn: uci_get(&format!("sqm.{}.ingress_ecn", section), "ECN"),
        egress_ecn: uci_get(&format!("sqm.{}.egress_ecn", section), "NOECN"),
        qdisc_really_really_advanced: uci_get(&format!("sqm.{}.qdisc_really_really_advanced", section), "false"),
        ilimit: uci_get(&format!("sqm.{}.ilimit", section), ""),
        elimit: uci_get(&format!("sqm.{}.elimit", section), ""),
        itarget: uci_get(&format!("sqm.{}.itarget", section), ""),
        etarget: uci_get(&format!("sqm.{}.etarget", section), ""),
        iqdisc_opts: uci_get(&format!("sqm.{}.iqdisc_opts", section), ""),
        eqdisc_opts: uci_get(&format!("sqm.{}.eqdisc_opts", section), ""),
        linklayer: uci_get(&format!("sqm.{}.linklayer", section), "none"),
        overhead: uci_get(&format!("sqm.{}.overhead", section), "0"),
        linklayer_advanced: uci_get(&format!("sqm.{}.linklayer_advanced", section), ""),
        tcMTU: uci_get(&format!("sqm.{}.tcMTU", section), "2047"),
        tcTSIZE: uci_get(&format!("sqm.{}.tcTSIZE", section), "128"),
        tcMPU: uci_get(&format!("sqm.{}.tcMPU", section), "0"),
        linklayer_adaptation_mechanism: uci_get(&format!("sqm.{}.linklayer_adaptation_mechanism", section), "default"),
    }
}

pub fn write_sqm_config(form: &SqmForm) -> Result<()> {
    let section = get_sqm_section();
    uci_set(&format!("{}.{}.enabled", "sqm", section), if form.enabled.is_some() { "1" } else { "0" })?;
    uci_set(&format!("{}.{}.download", "sqm", section), &form.download)?;
    uci_set(&format!("{}.{}.upload", "sqm", section), &form.upload)?;
    uci_set(&format!("{}.{}.debug_logging", "sqm", section), if form.debug_logging.is_some() { "1" } else { "0" })?;
    uci_set(&format!("{}.{}.verbosity", "sqm", section), &form.verbosity)?;
    uci_set(&format!("{}.{}.qdisc", "sqm", section), &form.qdisc)?;
    uci_set(&format!("{}.{}.script", "sqm", section), &form.script)?;
    uci_set(&format!("{}.{}.qdisc_advanced", "sqm", section), if form.qdisc_advanced.is_some() { "1" } else { "0" })?;
    uci_set(&format!("{}.{}.use_mq", "sqm", section), if form.use_mq.is_some() { "1" } else { "0" })?;
    uci_set(&format!("{}.{}.squash_dscp", "sqm", section), &form.squash_dscp)?;
    uci_set(&format!("{}.{}.squash_ingress", "sqm", section), &form.squash_ingress)?;
    uci_set(&format!("{}.{}.ingress_ecn", "sqm", section), &form.ingress_ecn)?;
    uci_set(&format!("{}.{}.egress_ecn", "sqm", section), &form.egress_ecn)?;
    uci_set(&format!("{}.{}.qdisc_really_really_advanced", "sqm", section), if form.qdisc_really_really_advanced.is_some() { "1" } else { "0" })?;
    uci_set(&format!("{}.{}.ilimit", "sqm", section), &form.ilimit)?;
    uci_set(&format!("{}.{}.elimit", "sqm", section), &form.elimit)?;
    uci_set(&format!("{}.{}.itarget", "sqm", section), &form.itarget)?;
    uci_set(&format!("{}.{}.etarget", "sqm", section), &form.etarget)?;
    uci_set(&format!("{}.{}.iqdisc_opts", "sqm", section), &form.iqdisc_opts)?;
    uci_set(&format!("{}.{}.eqdisc_opts", "sqm", section), &form.eqdisc_opts)?;
    uci_set(&format!("{}.{}.linklayer", "sqm", section), &form.linklayer)?;
    uci_set(&format!("{}.{}.overhead", "sqm", section), &form.overhead)?;
    uci_set(&format!("{}.{}.linklayer_advanced", "sqm", section), if form.linklayer_advanced.is_some() { "1" } else { "0" })?;
    uci_set(&format!("{}.{}.tcMTU", "sqm", section), &form.tcMTU)?;
    uci_set(&format!("{}.{}.tcTSIZE", "sqm", section), &form.tcTSIZE)?;
    uci_set(&format!("{}.{}.tcMPU", "sqm", section), &form.tcMPU)?;
    uci_set(&format!("{}.{}.linklayer_adaptation_mechanism", "sqm", section), &form.linklayer_adaptation_mechanism)?;

    // TODO: service restart for sqm
    std::process::Command::new("uci").args(["commit", "sqm"]).status()?;
    Ok(())
}

fn uci_get(key: &str, default: &str) -> String {
    std::process::Command::new("uci").args(["get", key]).output()
        .map(|o| { let s = String::from_utf8_lossy(&o.stdout).trim().to_string(); if s.is_empty() { default.to_string() } else { s } })
        .unwrap_or_else(|_| default.to_string())
}

fn uci_set(key: &str, value: &str) -> Result<()> {
    std::process::Command::new("uci").args(["set", &format!("{}={}", key, value)]).status()?;
    Ok(())
}
