#!/bin/bash

# --- 颜色定义 ---
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # 无颜色

# --- 配置区 ---
APP_NAME="pve-pci-fixer"
INSTALL_DIR="/usr/local/bin"
CONF_DIR="/etc/pve-pci-fixer"
# 已修正为你的真实 v0.1.0 Tag 地址
BINARY_URL="https://github.com/jeff141/pve-pci-fixer/releases/download/v0.1.0/pve-pci-fixer"
SERVICE_PATH="/etc/systemd/system/$APP_NAME.service"

echo -e "${BLUE}>>> Starting PvePciFixer Installation...${NC}"

# 1. 权限检查
if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}[Error]${NC} 请使用 root 权限运行"
  exit 1
fi

# 2. 创建目录
mkdir -p $CONF_DIR

# 3. 下载并赋权
echo -e "${BLUE}[1/4]${NC} Downloading binary from GitHub (v0.1.0)..."
curl -L $BINARY_URL -o $INSTALL_DIR/$APP_NAME
if [ $? -ne 0 ]; then
    echo -e "${RED}[Error]${NC} 下载失败，请检查网络或 URL 是否有效"
    exit 1
fi

# 检查下载的文件大小，防止下载到 404 文本
FILE_SIZE=$(stat -c%s "$INSTALL_DIR/$APP_NAME")
if [ $FILE_SIZE -lt 1000 ]; then
    echo -e "${RED}[Error]${NC} 下载的文件异常小 ($FILE_SIZE bytes)，可能是链接已失效。"
    exit 1
fi

chmod +x $INSTALL_DIR/$APP_NAME

# 4. 初始化默认配置
echo -e "${BLUE}[2/4]${NC} Creating default config.yml..."
if [ ! -f "$CONF_DIR/config.yml" ]; then
    cat <<EOF > $CONF_DIR/config.yml
mode: "prod"
enable_web: true
targets: []
EOF
fi

# 5. 配置 Systemd 服务
echo -e "${BLUE}[3/4]${NC} Configuring Systemd service..."
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
echo -e "${BLUE}[4/4]${NC} Enabling and starting service..."
systemctl daemon-reload
systemctl enable $APP_NAME
systemctl restart $APP_NAME

echo -e "${GREEN}------------------------------------------------${NC}"
echo -e "${GREEN}  Installation Successful! / 安装成功！${NC}"
echo -e "  Web UI: http://$(hostname -I | awk '{print $1}'):3000"
echo -e "  Config: $CONF_DIR/config.yml"
echo -e "${GREEN}------------------------------------------------${NC}"