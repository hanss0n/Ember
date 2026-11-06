# Installs for the current user (no sudo needed).
bin_dir := env_var('HOME') / ".local/bin"
app_dir := env_var('HOME') / ".local/share/applications"
app_id := "dev.bear.CosmicEmber"
applet_id := "dev.bear.CosmicAppletEmber"

default: build

build:
    cargo build --release

test:
    cargo test --release

check:
    cargo clippy --release --all-targets -- -D warnings

install: build
    install -Dm755 target/release/cosmic-ember {{bin_dir}}/cosmic-ember
    install -Dm755 target/release/cosmic-applet-ember {{bin_dir}}/cosmic-applet-ember
    install -Dm755 cli/ember {{bin_dir}}/ember
    install -Dm644 res/{{app_id}}.desktop {{app_dir}}/{{app_id}}.desktop
    install -Dm644 res/{{applet_id}}.desktop {{app_dir}}/{{applet_id}}.desktop
    # The panel may not have ~/.local/bin on PATH, so use absolute paths.
    sed -i 's|^Exec=|Exec={{bin_dir}}/|' {{app_dir}}/{{app_id}}.desktop {{app_dir}}/{{applet_id}}.desktop

uninstall:
    -{{bin_dir}}/ember timer off
    -{{bin_dir}}/ember day
    rm -f {{bin_dir}}/cosmic-ember {{bin_dir}}/cosmic-applet-ember {{bin_dir}}/ember
    rm -f {{app_dir}}/{{app_id}}.desktop {{app_dir}}/{{applet_id}}.desktop
