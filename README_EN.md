# PvePciFixer 🚀

English | [中文说明](./README.md)

**Author: 一坨肉 (OneTuRou)**

PvePciFixer is an automation utility designed for Proxmox VE (PVE) users. It solves the "PCI Address Drift" issue caused by adding or removing hardware (e.g., OCuLink devices, GPUs), which often prevents VMs from starting due to mismatched BDF addresses.

### Core Features
- ✅ **Dynamic Tracking**: Identifies hardware via name keywords instead of static BDF addresses.
- ✅ **Web Interface**: User-friendly UI to select VMs and PCI devices—no manual YAML editing required.
- ✅ **Boot-time Auto-Fix**: Run as a Systemd service to automatically realign BDF addresses during host startup.
- ✅ **Zero Dependencies**: Written in Rust; distributed as a single lightweight binary that keeps your host OS clean.

### Quick Start
1. Download the latest binary from [Releases](../../releases).
2. Grant execution permissions and run: `chmod +x pve-pci-fixer && ./pve-pci-fixer`.
3. Open your browser at `http://<PVE-IP>:3000` to bind your devices.

### Environment Modes
You can switch between `prod` (for real PVE hosts) and `dev` (for local testing with `lspci.txt`) by modifying the `mode` field in `config.yml`.