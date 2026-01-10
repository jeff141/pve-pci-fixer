#!/bin/bash

# --- 配置区 ---
APP_NAME="pve-pci-fixer"
INSTALL_DIR="/usr/local/bin"
CONF_DIR="/etc/pve-pci-fixer"
# 使用你提供的 Release 地址
BINARY_URL="https://github.com/jeff141/pve-pci-fixer/releases/download/version/pve-pci-fixer"
SERVICE_PATH="/etc/systemd/system/$APP_NAME.service"

echo "🚀 Starting PvePciFixer Installation..."

# 1. 环境检查
if [ "$EUID" -ne 0 ]; then
  echo "❌ 请使用 root 权限运行（PVE 宿主机通常默认为 root）"
  exit 1
fi

# 2. 创建目录
mkdir -p $CONF_DIR

# 3. 下载并赋权
echo "📥 Downloading binary from GitHub..."
curl -L $BINARY_URL -o $INSTALL_DIR/$APP_NAME
if [ $? -ne 0 ]; then
    echo "❌ 下载失败，请检查网络（或代理设置）"
    exit 1
fi
chmod +x $INSTALL_DIR/$APP_NAME

# 4. 初始化默认配置 (不覆盖已有配置)
if [ ! -f "$CONF_DIR/config.yml" ]; then
    echo "📝 Creating default config.yml..."
    cat <<EOF > $CONF_DIR/config.yml
mode: "prod"
enable_web: true
targets: []
EOF
fi

# 5. 配置 Systemd 服务 (实现开机自愈)
echo "⚙️ Configuring Systemd service..."
cat <<EOF > $SERVICE_PATH
[Unit]
Description=PVE PCI Address Fixer Service
After=network.target

[Service]
Type=simple
WorkingDirectory=$CONF_DIR
ExecStart=$INSTALL_DIR/$APP_NAME
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# 6. 激活服务
systemctl daemon-reload
systemctl enable $APP_NAME
systemctl restart $APP_NAME

echo "------------------------------------------------"
echo "✅ Installation Successful! / 安装成功！"
echo "🌐 Web UI: http://$(hostname -I | awk '{print $1}'):3000"
echo "📄 Config: $CONF_DIR/config.yml"
echo "------------------------------------------------"