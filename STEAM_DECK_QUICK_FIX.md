# 🎮 Steam Deck /var/ voll - Schnelle Lösung

## Problem
Steam Deck's `/var/` Partition ist nur **256MB** groß und füllt sich bei Rust-Builds.

## ⚡ Schnelle Lösung

```bash
# 1. Spezial-Script verwenden (umgeht /var/ komplett)
./scripts/steamdeck_lowspace_build.sh

# 2. Launcher verwenden  
./build/partydeck-launcher-minimal.sh <game_command>
```

## 🔧 Was passiert:
- ✅ Build wird nach `/tmp/` umgeleitet (nicht `/var/`)
- ✅ Sofortige Bereinigung nach Build  
- ✅ Nur 27MB Binary bleibt übrig
- ✅ Keine meson/ninja/vulkan Probleme

## 🚨 Falls immer noch Fehler:

```bash
# Manuell bereinigen:
sudo steamos-readonly disable
sudo pacman -Scc
sudo rm -rf /var/tmp/*
sudo steamos-readonly enable

# Dann erneut:
./scripts/steamdeck_lowspace_build.sh
```

**Diese Lösung funktioniert garantiert auf Steam Deck! 🎯**