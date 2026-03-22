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

pub fn read_sqm_config() -> SqmConfig {
    SqmConfig {
        enabled: uci_get("sqm.@sqm[0].enabled", ""),
        download: uci_get("sqm.@sqm[0].download", ""),
        upload: uci_get("sqm.@sqm[0].upload", ""),
        debug_logging: uci_get("sqm.@sqm[0].debug_logging", ""),
        verbosity: uci_get("sqm.@sqm[0].verbosity", "5"),
        qdisc: uci_get("sqm.@sqm[0].qdisc", "cake"),
        script: uci_get("sqm.@sqm[0].script", "piece_of_cake.qos"),
        qdisc_advanced: uci_get("sqm.@sqm[0].qdisc_advanced", "false"),
        use_mq: uci_get("sqm.@sqm[0].use_mq", "false"),
        squash_dscp: uci_get("sqm.@sqm[0].squash_dscp", "1"),
        squash_ingress: uci_get("sqm.@sqm[0].squash_ingress", "1"),
        ingress_ecn: uci_get("sqm.@sqm[0].ingress_ecn", "ECN"),
        egress_ecn: uci_get("sqm.@sqm[0].egress_ecn", "NOECN"),
        qdisc_really_really_advanced: uci_get("sqm.@sqm[0].qdisc_really_really_advanced", "false"),
        ilimit: uci_get("sqm.@sqm[0].ilimit", ""),
        elimit: uci_get("sqm.@sqm[0].elimit", ""),
        itarget: uci_get("sqm.@sqm[0].itarget", ""),
        etarget: uci_get("sqm.@sqm[0].etarget", ""),
        iqdisc_opts: uci_get("sqm.@sqm[0].iqdisc_opts", ""),
        eqdisc_opts: uci_get("sqm.@sqm[0].eqdisc_opts", ""),
        linklayer: uci_get("sqm.@sqm[0].linklayer", "none"),
        overhead: uci_get("sqm.@sqm[0].overhead", "0"),
        linklayer_advanced: uci_get("sqm.@sqm[0].linklayer_advanced", ""),
        tcMTU: uci_get("sqm.@sqm[0].tcMTU", "2047"),
        tcTSIZE: uci_get("sqm.@sqm[0].tcTSIZE", "128"),
        tcMPU: uci_get("sqm.@sqm[0].tcMPU", "0"),
        linklayer_adaptation_mechanism: uci_get("sqm.@sqm[0].linklayer_adaptation_mechanism", "default"),
    }
}

pub fn write_sqm_config(form: &SqmForm) -> Result<()> {
    uci_set("sqm.@sqm[0].enabled", if form.enabled.is_some() { "1" } else { "0" })?;
    uci_set("sqm.@sqm[0].download", &form.download)?;
    uci_set("sqm.@sqm[0].upload", &form.upload)?;
    uci_set("sqm.@sqm[0].debug_logging", if form.debug_logging.is_some() { "1" } else { "0" })?;
    uci_set("sqm.@sqm[0].verbosity", &form.verbosity)?;
    uci_set("sqm.@sqm[0].qdisc", &form.qdisc)?;
    uci_set("sqm.@sqm[0].script", &form.script)?;
    uci_set("sqm.@sqm[0].qdisc_advanced", if form.qdisc_advanced.is_some() { "1" } else { "0" })?;
    uci_set("sqm.@sqm[0].use_mq", if form.use_mq.is_some() { "1" } else { "0" })?;
    uci_set("sqm.@sqm[0].squash_dscp", &form.squash_dscp)?;
    uci_set("sqm.@sqm[0].squash_ingress", &form.squash_ingress)?;
    uci_set("sqm.@sqm[0].ingress_ecn", &form.ingress_ecn)?;
    uci_set("sqm.@sqm[0].egress_ecn", &form.egress_ecn)?;
    uci_set("sqm.@sqm[0].qdisc_really_really_advanced", if form.qdisc_really_really_advanced.is_some() { "1" } else { "0" })?;
    uci_set("sqm.@sqm[0].ilimit", &form.ilimit)?;
    uci_set("sqm.@sqm[0].elimit", &form.elimit)?;
    uci_set("sqm.@sqm[0].itarget", &form.itarget)?;
    uci_set("sqm.@sqm[0].etarget", &form.etarget)?;
    uci_set("sqm.@sqm[0].iqdisc_opts", &form.iqdisc_opts)?;
    uci_set("sqm.@sqm[0].eqdisc_opts", &form.eqdisc_opts)?;
    uci_set("sqm.@sqm[0].linklayer", &form.linklayer)?;
    uci_set("sqm.@sqm[0].overhead", &form.overhead)?;
    uci_set("sqm.@sqm[0].linklayer_advanced", if form.linklayer_advanced.is_some() { "1" } else { "0" })?;
    uci_set("sqm.@sqm[0].tcMTU", &form.tcMTU)?;
    uci_set("sqm.@sqm[0].tcTSIZE", &form.tcTSIZE)?;
    uci_set("sqm.@sqm[0].tcMPU", &form.tcMPU)?;
    uci_set("sqm.@sqm[0].linklayer_adaptation_mechanism", &form.linklayer_adaptation_mechanism)?;

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
