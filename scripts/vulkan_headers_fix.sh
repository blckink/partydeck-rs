#!/usr/bin/env bash

# 🔧 Vulkan Headers Fix für Steam Deck
# Löst "Missing vulkan-headers" Meson Build Fehler

set -e

echo "🔧 Vulkan Headers Fix für Steam Deck wird angewendet..."

# Vulkan Headers Pfade für Steam Deck
export VULKAN_SDK="/usr"
export VK_SDK_PATH="/usr"
export VULKAN_INCLUDE_DIR="/usr/include/vulkan"
export VK_LAYER_PATH="/usr/share/vulkan/explicit_layer.d"

# C/C++ Include Pfade für Vulkan
export CPATH="/usr/include/vulkan:$CPATH"
export C_INCLUDE_PATH="/usr/include/vulkan:$C_INCLUDE_PATH" 
export CPLUS_INCLUDE_PATH="/usr/include/vulkan:$CPLUS_INCLUDE_PATH"

# PKG_CONFIG_PATH erweitern
export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:$PKG_CONFIG_PATH"

# Meson Environment
export MESON_TESTLOG_JUNIT="/tmp/testlog.xml"

# Prüfe ob Vulkan Headers verfügbar sind
if [[ -f "/usr/include/vulkan/vulkan.h" ]]; then
    echo "✅ Vulkan Headers gefunden: /usr/include/vulkan/vulkan.h"
else
    echo "❌ Vulkan Headers nicht gefunden!"
    
    # Versuche Installation über pacman (SteamOS)
    if command -v pacman &> /dev/null; then
        echo "🔄 Installiere Vulkan Headers über pacman..."
        sudo pacman -S --noconfirm vulkan-headers 2>/dev/null || echo "⚠️ pacman Installation fehlgeschlagen"
    fi
    
    # Versuche Installation über apt (Debian/Ubuntu)
    if command -v apt &> /dev/null; then
        echo "🔄 Installiere Vulkan Headers über apt..."
        sudo apt update && sudo apt install -y libvulkan-dev vulkan-headers 2>/dev/null || echo "⚠️ apt Installation fehlgeschlagen"
    fi
fi

# Erstelle vulkan.pc falls nicht vorhanden
VULKAN_PC="/usr/lib/pkgconfig/vulkan.pc"
if [[ ! -f "$VULKAN_PC" ]]; then
    echo "🔧 Erstelle vulkan.pc für pkg-config..."
    sudo mkdir -p "$(dirname "$VULKAN_PC")"
    
    sudo tee "$VULKAN_PC" > /dev/null << 'EOF'
prefix=/usr
exec_prefix=${prefix}
includedir=${prefix}/include
libdir=${exec_prefix}/lib

Name: Vulkan-Headers
Description: Vulkan Header files and API registry
Version: 1.3.0
Cflags: -I${includedir}/vulkan
EOF
    
    echo "✅ vulkan.pc erstellt: $VULKAN_PC"
fi

# Teste pkg-config
if pkg-config --exists vulkan; then
    echo "✅ pkg-config findet Vulkan: $(pkg-config --modversion vulkan)"
    echo "✅ Vulkan CFLAGS: $(pkg-config --cflags vulkan)"
else
    echo "⚠️ pkg-config kann Vulkan nicht finden"
fi

# Exportiere alle Variablen für nachfolgende Builds
cat > /tmp/vulkan_env.sh << EOF
export VULKAN_SDK="$VULKAN_SDK"
export VK_SDK_PATH="$VK_SDK_PATH"
export VULKAN_INCLUDE_DIR="$VULKAN_INCLUDE_DIR"
export VK_LAYER_PATH="$VK_LAYER_PATH"
export CPATH="$CPATH"
export C_INCLUDE_PATH="$C_INCLUDE_PATH"
export CPLUS_INCLUDE_PATH="$CPLUS_INCLUDE_PATH"
export PKG_CONFIG_PATH="$PKG_CONFIG_PATH"
EOF

echo "🎯 Vulkan Environment gespeichert in: /tmp/vulkan_env.sh"
echo "💡 Verwende: source /tmp/vulkan_env.sh vor dem Build"
echo "🔧 Vulkan Headers Fix komplett!"