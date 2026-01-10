# PvePciFixer 🚀

English | [中文说明](./README.md)

**Author: yiTuoRou (一坨肉)**

A lightweight automation tool for Proxmox VE (PVE) users to solve "PCI BDF Drift." It ensures your VMs always find the right hardware (GPU, OCuLink, NVMe) even after hardware changes re-enumerate the PCI bus.

### Core Features
- ✅ **Dynamic Binding**: Tracks hardware by name keywords instead of fragile static BDF addresses.
- ✅ **Web UI**: Modern management panel for easy VM-to-Hardware binding.
- ✅ **Auto-Fix on Boot**: Systemd integration ensures all BDFs are realigned during host startup.
- ✅ **Strict Compliance**: Automatically handles the `0000:` domain prefix for PVE config standard.
- ✅ **Pure Rust**: High performance, zero dependencies, and a single binary that keeps your host OS clean.

### Quick Install (Recommended)
Run the following command in your PVE terminal to install and enable the service automatically:
```bash
curl -sSL [https://raw.githubusercontent.com/jeff141/pve-pci-fixer/master/install.sh](https://raw.githubusercontent.com/jeff141/pve-pci-fixer/master/install.sh) | bash