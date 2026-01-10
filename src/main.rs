/*
 * PvePciFixer - A Proxmox VE PCI BDF Drift Fixer
 * Copyright (C) 2026 一坨肉 (OneTuRou)
 * * This program is licensed under the MIT License.
 * GitHub: https://github.com/jeff141/pve-pci-fixer
 */
use axum::{routing::{get, post}, Json, Router};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::process::Command;

// --- 数据结构定义 ---
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    #[serde(default = "default_mode")]
    mode: String, // "prod" 或 "dev"
    #[serde(default = "default_enable_web")]
    enable_web: bool,
    targets: Vec<Target>,
}

fn default_mode() -> String { "prod".to_string() }
fn default_enable_web() -> bool { true }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Target {
    vmid: u32,
    pci_slots: Vec<PciSlot>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PciSlot {
    key: String,
    name_keyword: String,
}

#[derive(Serialize)]
struct VMInfo {
    vmid: u32,
    name: String,
}

#[derive(Deserialize)]
struct BindRequest {
    vmid: u32,
    name_keyword: String,
    key: String, // 如果为空，后端将自动计算 hostpciN
}

// --- 常量配置 ---
const CONFIG_YML: &str = "config.yml";
const DEV_LSPCI_FILE: &str = "lspci.txt";
const DEV_CONF_DIR: &str = "/home/jeff/fsdownload/qemu-server";
const PROD_CONF_DIR: &str = "/etc/pve/qemu-server";

// --- PCI 列表获取逻辑 (环境隔离) ---

fn parse_pci_lines(content: &str) -> Vec<(String, String)> {
    content.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
            } else { None }
        })
        .collect()
}

fn get_system_pci_list_dev() -> Vec<(String, String)> {
    let content = fs::read_to_string(DEV_LSPCI_FILE).expect("DEV模式错误：找不到 lspci.txt");
    parse_pci_lines(&content)
}

fn get_system_pci_list_prod() -> Vec<(String, String)> {
    let output = Command::new("lspci").output().expect("PROD模式错误：执行 lspci 失败");
    let content = String::from_utf8_lossy(&output.stdout);
    parse_pci_lines(&content)
}

fn get_current_pci_list(config: &Config) -> Vec<(String, String)> {
    if config.mode == "dev" {
        get_system_pci_list_dev()
    } else {
        get_system_pci_list_prod()
    }
}

// --- 核心修复逻辑 ---

fn do_fix_logic() -> String {
    let mut log = String::new();
    if !std::path::Path::new(CONFIG_YML).exists() {
        return "等待配置：config.yml 尚不存在。".to_string();
    }

    let config_str = fs::read_to_string(CONFIG_YML).expect("读取 YAML 失败");
    let config: Config = serde_yaml::from_str(&config_str).expect("解析 YAML 失败");

    let current_pci_map = get_current_pci_list(&config);
    let base_path = if config.mode == "dev" { DEV_CONF_DIR } else { PROD_CONF_DIR };

    log.push_str(&format!(">>> 当前模式: {}\n", config.mode));

    for target in config.targets {
        let conf_path = format!("{}/{}.conf", base_path, target.vmid);
        if !std::path::Path::new(&conf_path).exists() {
            log.push_str(&format!("跳过 VM {}: 配置文件不存在\n", target.vmid));
            continue;
        }

        let mut conf_content = fs::read_to_string(&conf_path).expect("读取 PVE 配置失败");
        let mut changed = false;

        for slot in target.pci_slots {
            if let Some(mut new_addr) = current_pci_map.iter()
                .find(|(_, name)| name.contains(&slot.name_keyword))
                .map(|(addr, _)| addr.clone())
            {
                // ✨ 核心修改：如果地址不包含冒号分隔的 Domain (即长度较短)，自动补齐 0000:
                if !new_addr.contains("0000:") && new_addr.matches(':').count() == 1 {
                    new_addr = format!("0000:{}", new_addr);
                }

                let pattern = format!(r"(?m)^({}:\s*)(?:0000:)?([0-9a-fA-F:\.]+)(.*)$", slot.key);
                let re = Regex::new(&pattern).unwrap();

                if let Some(caps) = re.captures(&conf_content) {
                    let old_addr = &caps[2];
                    let suffix = &caps[3];

                    // 统一对比格式：确保 old_addr 如果没有前缀也补上，或者 new_addr 剥离后对比
                    // 这里我们采用最稳妥的方法：直接对比最终生成的字符串
                    if old_addr != new_addr && format!("0000:{}", old_addr) != new_addr {
                        log.push_str(&format!("VM {}: {} 修正 [{} -> {}]\n", target.vmid, slot.key, old_addr, new_addr));
                        let replacement = format!("{}{}{}", &caps[1], new_addr, suffix);
                        conf_content = re.replace(&conf_content, replacement.as_str()).to_string();
                        changed = true;
                    }
                }
            }
        }

        if changed {
            fs::write(&conf_path, conf_content).expect("写入 PVE 配置失败");
            log.push_str(&format!("VM {}: 配置已更新并同步磁盘\n", target.vmid));
        }
    }
    if log.lines().count() <= 1 { "所有设备状态正常，未检测到漂移。".to_string() } else { log }
}

// --- Web 接口实现 ---

async fn api_get_vms() -> Json<Vec<VMInfo>> {
    let config_str = fs::read_to_string(CONFIG_YML).unwrap_or_else(|_| "mode: prod".to_string());
    let config: Config = serde_yaml::from_str(&config_str).unwrap_or(Config { mode: "prod".into(), enable_web: true, targets: vec![] });
    let base_path = if config.mode == "dev" { DEV_CONF_DIR } else { PROD_CONF_DIR };

    let mut vms = Vec::new();
    if let Ok(entries) = fs::read_dir(base_path) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("conf") {
                let vmid_str = entry.path().file_stem().unwrap().to_str().unwrap().to_string();
                if let Ok(vmid) = vmid_str.parse::<u32>() {
                    let content = fs::read_to_string(entry.path()).unwrap_or_default();
                    let name = content.lines()
                        .find(|l| l.starts_with("name:"))
                        .map(|l| l.replace("name:", "").trim().to_string())
                        .unwrap_or(format!("VM {}", vmid));
                    vms.push(VMInfo { vmid, name });
                }
            }
        }
    }
    Json(vms)
}

async fn api_save_bind(Json(payload): Json<BindRequest>) -> String {
    let mut config: Config = if let Ok(s) = fs::read_to_string(CONFIG_YML) {
        serde_yaml::from_str(&s).unwrap_or(Config { mode: "prod".into(), enable_web: true, targets: vec![] })
    } else {
        Config { mode: "prod".into(), enable_web: true, targets: vec![] }
    };

    let base_path = if config.mode == "dev" { DEV_CONF_DIR } else { PROD_CONF_DIR };
    let conf_path = format!("{}/{}.conf", base_path, payload.vmid);

    if !std::path::Path::new(&conf_path).exists() {
        return "错误：找不到该虚拟机的配置文件".into();
    }

    let conf_content = fs::read_to_string(&conf_path).unwrap_or_default();

    // 自动计算槽位
    let final_key = if payload.key.trim().is_empty() {
        let re = Regex::new(r"hostpci(\d+):").unwrap();
        let mut max_idx = -1;
        for cap in re.captures_iter(&conf_content) {
            if let Ok(idx) = cap[1].parse::<i32>() { if idx > max_idx { max_idx = idx; } }
        }
        format!("hostpci{}", max_idx + 1)
    } else {
        payload.key.trim().to_string()
    };

    // 更新内存中的配置
    let target = config.targets.iter_mut().find(|t| t.vmid == payload.vmid);
    let new_slot = PciSlot { key: final_key.clone(), name_keyword: payload.name_keyword };
    match target {
        Some(t) => {
            t.pci_slots.retain(|s| s.key != new_slot.key);
            t.pci_slots.push(new_slot);
        },
        None => {
            config.targets.push(Target { vmid: payload.vmid, pci_slots: vec![new_slot] });
        }
    }

    // 保存 YAML
    fs::write(CONFIG_YML, serde_yaml::to_string(&config).unwrap()).unwrap();

    // 如果配置文件里没这一行，初始化它
    if !conf_content.contains(&format!("{}:", final_key)) {
        let mut new_conf = conf_content;
        new_conf.push_str(&format!("\n{}: 00:00.0,pcie=1\n", final_key));
        fs::write(&conf_path, new_conf).unwrap();
    }

    format!("成功绑定至 {}，请点击‘执行修正’同步地址。", final_key)
}

// --- 主函数 ---

#[tokio::main]
async fn main() {
    println!(">>> PvePciFixer 正在启动...");

    // 启动即执行一次修复
    println!("{}", do_fix_logic());

    let app = Router::new()
        // ✅ 修正：只保留这一个 /api/pci 处理逻辑
        .route("/api/pci", get(|| async {
            let s = fs::read_to_string(CONFIG_YML).unwrap_or_else(|_| "mode: prod".into());
            let c: Config = serde_yaml::from_str(&s).unwrap_or(Config { mode: "prod".into(), enable_web: true, targets: vec![] });
            Json(get_current_pci_list(&c))
        }))
        .route("/api/vms", get(api_get_vms))
        .route("/api/fix", get(|| async { do_fix_logic() }))
        .route("/api/save", post(api_save_bind))
        .route("/", get(|| async {
            axum::response::Html(include_str!("index.html"))
        }));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!(">>> 管理面板已就绪: http://localhost:3000");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}