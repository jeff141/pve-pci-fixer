use std::process::Command;
use std::fs;
use regex::Regex;

fn main() {
    let mapping_path = "/etc/pve/mapping/pci.cfg";

        // 1. 执行 lspci -D (相当于 Java 的 ProcessBuilder)
    let lspci_out = Command::new("lspci")
        .arg("-D")
        .output()
        .expect("无法执行 lspci");
    let lspci_data = String::from_utf8_lossy(&lspci_out.stdout);
    
}