# 🎮 PARTYDECK-RS - STEAM DECK NATIVE EDITION

## 🎯 Mission: Steam Deck Split-Screen Monster

**Scheiß auf andere Systeme!** 😂 Wir optimieren ALLES für Steam Deck:

### 🚀 Steam Deck Native Features:

1. **Multiple Mäuse + Keyboards** gleichzeitig im Split-Screen
2. **Handheld + Docked Mode** optimiert
3. **Steam Input Integration** (Steam Controller API)
4. **SteamOS Wayland** nativ
5. **Steam Deck Hardware** maximal ausnutzen (APU, RAM, etc.)
6. **gamescope Integration** tief ins System
7. **Steam Overlay** kompatibel
8. **Power Management** optimiert

### 🖱️ Multi-Device Split-Screen Konzept:

```
┌─────────────────────────────────────────┐
│           Steam Deck Display            │
│  ┌─────────────────┬─────────────────┐  │
│  │   Player 1      │   Player 2      │  │
│  │  🖱️ Maus 1      │  🖱️ Maus 2      │  │
│  │  ⌨️ Keyboard 1  │  ⌨️ Keyboard 2  │  │
│  │                 │                 │  │
│  └─────────────────┴─────────────────┘  │
│  ┌─────────────────┬─────────────────┐  │
│  │   Player 3      │   Player 4      │  │
│  │  🎮 Steam Deck  │  🎮 Pro Ctrl   │  │
│  │                 │                 │  │
│  └─────────────────┴─────────────────┘  │
└─────────────────────────────────────────┘
```

### 🔧 Steam Deck Optimierungen:

#### Hardware-Spezifisch:
- **AMD APU** optimierte Grafik-Pipeline
- **16GB RAM** intelligent aufteilen für 4 Game-Instanzen
- **7" 1280x800** Display perfekt für 2x2 Split-Screen
- **Steam Deck Gamepad** als primären Controller behandeln

#### Software-Spezifisch:
- **SteamOS 3.x** native Integration
- **Wayland/gamescope** direkte Anbindung
- **Steam Runtime** Container verwenden
- **KDE Plasma** (Steam Deck Desktop) Integration

### 🎮 Input-System Redesign:

#### Prioritäten:
1. **Steam Deck Built-in Controls** = Player 1 (immer)
2. **USB-Hub angeschlossen:**
   - Maus 1 + Keyboard 1 = Player 2
   - Maus 2 + Keyboard 2 = Player 3
   - USB Controller = Player 4

#### Steam Input API Integration:
```rust
// Steam Deck native input handling
use steam_input::*;

// Detect Steam Deck built-in controller
let steam_deck_controller = SteamInput::get_builtin_controller();

// Scan for additional USB devices
let usb_mice = scan_usb_mice();
let usb_keyboards = scan_usb_keyboards();
let usb_controllers = scan_usb_controllers();
```

### 🖥️ Display-Management:

#### Handheld Mode (1280x800):
```
┌─────────────────────────────────┐
│  640x400  │  640x400           │
│  Player1  │  Player2           │
├───────────┼────────────────────┤
│  640x400  │  640x400           │
│  Player3  │  Player4           │
└─────────────────────────────────┘
```

#### Docked Mode (bis 4K):
```
┌─────────────────────────────────┐
│  960x540  │  960x540           │
│  Player1  │  Player2           │
├───────────┼────────────────────┤
│  960x540  │  960x540           │
│  Player3  │  Player4           │
└─────────────────────────────────┘
```

### ⚡ Performance Optimierungen:

#### CPU (Zen 2):
- **4 Threads** für 4 Game-Instanzen
- **Low-latency scheduling** für Input
- **CPU Governor** auf Performance

#### GPU (RDNA 2):
- **Shared GPU Memory** intelligent aufteilen
- **Vulkan** für alle Rendering
- **FSR** für Performance-Boost

#### RAM:
- **4GB pro Game-Instanz** maximal
- **Shared Libraries** um RAM zu sparen
- **Memory-mapped files** für Assets

### 🎯 Steam Deck Features nutzen:

#### Hardware:
- **Gyroscope** für Bewegungssteuerung
- **Touchpads** als Maus-Alternative
- **Back-Buttons** für zusätzliche Controls
- **Haptic Feedback** für alle Player

#### Software:
- **Steam Overlay** für jeden Player separat
- **Steam Screenshots** von jedem Quadrant
- **Steam Cloud** für Saves
- **Steam Workshop** für Mods

### 🔨 Build-System Steam Deck Native:

```bash
# Komplett für Steam Deck optimiert
export STEAM_DECK_NATIVE=1
export TARGET_ARCH=x86_64-unknown-linux-gnu
export CPU_TARGET=znver2  # AMD Zen 2
export GPU_TARGET=gfx1030 # RDNA 2

# SteamOS spezifische Flags
export STEAMOS_VERSION=3.5
export USE_STEAM_RUNTIME=1
export ENABLE_STEAM_INPUT=1
```

Lass uns das Ding richtig Steam Deck-spezifisch machen! 🔥