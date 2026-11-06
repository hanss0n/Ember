# Installs for the current user (no sudo needed).
bin_dir := env_var('HOME') / ".local/bin"
app_dir := env_var('HOME') / ".local/share/applications"

default: build

build:
    cargo build --release

test:
    cargo test --release --lib

check:
    cargo clippy --release -- -D warnings

install: build
    install -Dm755 target/release/cosmic-nightlight {{bin_dir}}/cosmic-nightlight
    install -Dm755 target/release/cosmic-applet-nightlight {{bin_dir}}/cosmic-applet-nightlight
    install -Dm755 cli/nightlight {{bin_dir}}/nightlight
    install -Dm644 res/dev.bear.CosmicNightlight.desktop {{app_dir}}/dev.bear.CosmicNightlight.desktop
    install -Dm644 res/dev.bear.CosmicAppletNightlight.desktop {{app_dir}}/dev.bear.CosmicAppletNightlight.desktop
    # The panel may not have ~/.local/bin on PATH, so use absolute paths.
    sed -i 's|^Exec=|Exec={{bin_dir}}/|' {{app_dir}}/dev.bear.CosmicNightlight.desktop {{app_dir}}/dev.bear.CosmicAppletNightlight.desktop

uninstall:
    -{{bin_dir}}/nightlight timer off
    -{{bin_dir}}/nightlight day
    rm -f {{bin_dir}}/cosmic-nightlight {{bin_dir}}/cosmic-applet-nightlight {{bin_dir}}/nightlight
    rm -f {{app_dir}}/dev.bear.CosmicNightlight.desktop {{app_dir}}/dev.bear.CosmicAppletNightlight.desktop
