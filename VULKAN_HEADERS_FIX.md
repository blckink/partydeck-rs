# 🔧 Vulkan Headers Fix für Steam Deck

## Problem
```
meson.build:95:2: ERROR: Problem encountered: Missing vulkan-headers
```

## Ursache
Auf Steam Deck/SteamOS sind die Vulkan Headers zwar installiert (`/usr/include/vulkan/vulkan.h`), aber:
1. **pkg-config** findet sie nicht (fehlende `vulkan.pc`)
2. **Meson** kann die Include-Pfade nicht auflösen
3. **Environment Variables** sind nicht gesetzt

## 🚀 Automatische Lösung

### Mit Ultimate Build Script
```bash
./steamdeck_ultimate_build.sh
```
Das Script führt **automatisch** den Vulkan Fix aus!

### Manueller Fix
```bash
# Vulkan Fix Script ausführen
chmod +x scripts/vulkan_headers_fix.sh
./scripts/vulkan_headers_fix.sh

# Environment laden
source /tmp/vulkan_env.sh

# Jetzt sollte der Build funktionieren
cargo build --release
```

## 🔧 Was der Fix macht

### 1. Environment Variables setzen
```bash
export VULKAN_SDK="/usr"
export VK_SDK_PATH="/usr"
export VULKAN_INCLUDE_DIR="/usr/include/vulkan"
export VK_LAYER_PATH="/usr/share/vulkan/explicit_layer.d"
```

### 2. C/C++ Include Pfade
```bash
export CPATH="/usr/include/vulkan:$CPATH"
export C_INCLUDE_PATH="/usr/include/vulkan:$C_INCLUDE_PATH"
export CPLUS_INCLUDE_PATH="/usr/include/vulkan:$CPLUS_INCLUDE_PATH"
```

### 3. PKG_CONFIG_PATH erweitern
```bash
export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:$PKG_CONFIG_PATH"
```

### 4. vulkan.pc erstellen (falls nötig)
```ini
[/usr/lib/pkgconfig/vulkan.pc]
prefix=/usr
exec_prefix=${prefix}
includedir=${prefix}/include
libdir=${exec_prefix}/lib

Name: Vulkan-Headers
Description: Vulkan Header files and API registry
Version: 1.3.0
Cflags: -I${includedir}/vulkan
```

## ✅ Test ob Fix funktioniert

```bash
# Vulkan Headers vorhanden?
ls -la /usr/include/vulkan/vulkan.h

# pkg-config funktioniert?
pkg-config --exists vulkan && echo "✅ Vulkan gefunden" || echo "❌ Vulkan nicht gefunden"

# CFLAGS korrekt?
pkg-config --cflags vulkan
```

## 📁 Relevante Dateien

- `scripts/vulkan_headers_fix.sh` - Hauptfix Script
- `steamdeck_ultimate_build.sh` - Führt Fix automatisch aus
- `/tmp/vulkan_env.sh` - Gespeicherte Environment Variables

## 🎯 Steam Deck Spezifisch

Der Fix ist optimiert für:
- **SteamOS** (Arch Linux basiert)
- **Steam Deck APU** (AMD RDNA 2)
- **gamescope** (Wayland Compositor)
- **256MB /var/** (Limited Space)

## 💡 Tipps

1. **Immer zuerst** das Ultimate Build Script verwenden
2. Bei **manuellen Builds**: `source /tmp/vulkan_env.sh`
3. **Permanent machen**: Environment in `~/.bashrc` eintragen
4. **System Update**: Nach SteamOS Updates ggf. wiederholen

## 🔥 Für andere Systeme

Der Fix funktioniert auch auf:
- **Arch Linux** (pacman)
- **Ubuntu/Debian** (apt)
- **Generic Linux** (mit Vulkan Dev Packages)

**Scheiß auf andere Systeme - aber es funktioniert trotzdem!** 😂