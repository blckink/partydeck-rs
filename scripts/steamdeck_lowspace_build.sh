#!/usr/bin/env bash

# Steam Deck Low-Space Build Script
# Optimiert für Steam Deck's 256MB /var/ Partition

set -e

echo "💾 Steam Deck Low-Space Build (für 256MB /var/ Partition)..."
echo "============================================================"

# Function to check available space
check_space() {
    local path="$1"
    local required_mb="$2"
    local available=$(df -m "$path" | tail -1 | awk '{print $4}')
    
    echo "📊 Verfügbarer Speicher in $path: ${available}MB"
    
    if [[ $available -lt $required_mb ]]; then
        echo "⚠️  Warnung: Nur ${available}MB verfügbar, ${required_mb}MB empfohlen"
        return 1
    fi
    return 0
}

# Function to clean before build
clean_before_build() {
    echo "🧹 Bereinige vor Build..."
    
    # Clean old Rust artifacts
    if [[ -d "target" ]]; then
        rm -rf target/
        echo "✅ Alte target/ Verzeichnis entfernt"
    fi
    
    # Clean system cache if we have sudo
    if command -v sudo &> /dev/null; then
        sudo apt clean 2>/dev/null || true
        echo "✅ APT Cache bereinigt"
    fi
    
    # Clean home cargo cache
    if [[ -d "$HOME/.cargo/registry/cache" ]]; then
        rm -rf "$HOME/.cargo/registry/cache"/*
        echo "✅ Cargo Cache bereinigt"
    fi
    
    # Use tmpfs for cargo temp if possible
    export CARGO_TARGET_DIR="/tmp/partydeck-build"
    mkdir -p "$CARGO_TARGET_DIR"
    echo "✅ Build-Verzeichnis nach /tmp/ umgeleitet"
}

# Function for space-optimized build
build_optimized() {
    echo "🔨 Starte speicher-optimierten Build..."
    
    # Set up Rust environment
    if [[ -f "$HOME/.cargo/env" ]]; then
        source "$HOME/.cargo/env"
    elif [[ -f "/usr/local/cargo/env" ]]; then
        source "/usr/local/cargo/env"
    fi
    
    # Use system SSL libraries
    export OPENSSL_DIR=/usr
    export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
    export OPENSSL_INCLUDE_DIR=/usr/include/openssl
    export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:$PKG_CONFIG_PATH
    
    # Build with minimal profile to save space
    echo "🚀 Kompiliere mit --release (speicher-optimiert)..."
    cargo build --release
    
    if [[ $? -eq 0 ]]; then
        echo "✅ Build erfolgreich!"
        
        # Copy to build directory and clean temp
        mkdir -p build
        cp "$CARGO_TARGET_DIR/release/partydeck-rs" build/
        
        # Clean immediately to free space
        rm -rf "$CARGO_TARGET_DIR"
        
        echo "📁 Binary kopiert nach: build/partydeck-rs"
        echo "🧹 Temporäre Build-Dateien entfernt"
        
        return 0
    else
        echo "❌ Build fehlgeschlagen!"
        return 1
    fi
}

# Function to create space-efficient launcher
create_minimal_launcher() {
    echo "🚀 Erstelle minimalen Launcher..."
    
    mkdir -p build
    
    cat > build/partydeck-launcher-minimal.sh << 'EOF'
#!/usr/bin/env bash
# Minimaler Steam Deck Launcher

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY="$SCRIPT_DIR/partydeck-rs"

if [[ ! -f "$BINARY" ]]; then
    echo "❌ partydeck-rs binary nicht gefunden!"
    echo "💡 Bitte führe ./scripts/steamdeck_lowspace_build.sh aus"
    exit 1
fi

# Versuche gamescope zu finden
if command -v gamescope &> /dev/null; then
    echo "🎮 Starte mit gamescope..."
    exec gamescope --nested-width=1280 --nested-height=800 -- "$BINARY" "$@"
else
    echo "🖥️  Starte direkt (kein gamescope gefunden)..."
    exec "$BINARY" "$@"
fi
EOF
    
    chmod +x build/partydeck-launcher-minimal.sh
    echo "✅ Minimaler Launcher erstellt"
}

# Function for Steam Deck specific optimizations
setup_steamdeck_optimizations() {
    echo "⚙️  Steam Deck Optimierungen..."
    
    # Set Steam Deck environment variables
    export STEAMDECK=1
    export SDL_VIDEODRIVER=wayland
    
    # Use ramdisk for temp files if available
    if [[ -w "/dev/shm" ]]; then
        export TMPDIR="/dev/shm"
        echo "✅ Verwende RAM-Disk für temporäre Dateien"
    fi
    
    # Optimize for Steam Deck hardware
    export CARGO_BUILD_JOBS=4  # Steam Deck has 4 cores
    echo "✅ Build-Jobs auf 4 limitiert (Steam Deck CPU)"
}

# Main function
main() {
    echo "🎮 Steam Deck erkannt - verwende Low-Space Strategie"
    
    # Check if we're really on Steam Deck
    if [[ -f "/etc/os-release" ]] && grep -q "steamos" /etc/os-release; then
        echo "✅ SteamOS bestätigt"
        setup_steamdeck_optimizations
    else
        echo "ℹ️  Nicht auf SteamOS, aber verwende Low-Space Modus"
    fi
    
    # Check available space in critical locations
    echo "📊 Überprüfe verfügbaren Speicherplatz..."
    check_space "/var" 50 || echo "⚠️  /var/ ist fast voll - verwende alternative Strategie"
    check_space "/tmp" 100 || echo "⚠️  /tmp/ ist fast voll"
    check_space "." 50 || echo "⚠️  Aktuelles Verzeichnis hat wenig Platz"
    
    # Clean before build
    clean_before_build
    
    # Optimized build
    if build_optimized; then
        create_minimal_launcher
        
        echo ""
        echo "🎉 Steam Deck Low-Space Build erfolgreich!"
        echo ""
        echo "📦 Erstellt:"
        echo "   • build/partydeck-rs ($(du -h build/partydeck-rs 2>/dev/null | cut -f1 || echo '~27MB'))"
        echo "   • build/partydeck-launcher-minimal.sh"
        echo ""
        echo "🚀 Verwendung:"
        echo "   ./build/partydeck-launcher-minimal.sh <game_command>"
        echo ""
        echo "💡 Dieser Build verwendet minimal Speicherplatz und ist"
        echo "   speziell für Steam Deck's 256MB /var/ Partition optimiert."
        
    else
        echo "❌ Build fehlgeschlagen!"
        echo ""
        echo "🔧 Fehlerbehebung:"
        echo "1. Überprüfe verfügbaren Speicherplatz: df -h"
        echo "2. Bereinige manuell: sudo steamos-readonly disable && sudo pacman -Scc"
        echo "3. Neustart und erneut versuchen"
        exit 1
    fi
}

# Execute main function
main "$@"