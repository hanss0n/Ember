# cosmic-nightlight

Night light for the COSMIC desktop. It works by changing external monitors' own color
settings over DDC/CI, because COSMIC doesn't expose gamma control to apps like gammastep.

## Pieces

| Component | What it does |
|---|---|
| `cli/nightlight` | Python CLI that does the DDC/CI work through `ddcutil`. It also owns the systemd timer. |
| `cosmic-nightlight` | Settings app (libcosmic): mode, schedule, warmth, monitors, background timer. |
| `cosmic-applet-nightlight` | Panel applet: sun or moon icon, with a popup for switching modes. |

All three share `~/.config/nightlight/config.json`. The CLI records the level it last
applied in `$XDG_RUNTIME_DIR/nightlight.state`, which the GUI reads to show status.

## Source layout

```
src/
  lib.rs          shared crate root
  config.rs       Config / Mode / Monitor, JSON load + save
  schedule.rs     ClockTime and warmth-level math (with unit tests)
  backend.rs      calls into the CLI and systemd
  paths.rs        config, state and binary locations
  status.rs       status text and sun/moon icon choice
  bin/app/        settings app: main.rs, app.rs (state + update), view.rs, time_picker.rs
  bin/applet/     panel applet: main.rs, applet.rs (state + update), popup.rs
cli/nightlight    the CLI
res/              desktop entries
```

## Build and install (per user, no sudo)

Requires `ddcutil`, and your user in the `i2c` group.

```
just install      # build, then copy binaries, CLI and desktop files into ~/.local
just test         # unit tests
just check        # clippy with warnings as errors
just uninstall    # stop the timer, reset monitors to normal, remove everything
```

Then add the applet: **Settings → Desktop → Panel → Applets → Add applet → Night Light**.

## CLI

```
nightlight auto | night | day     set the mode
nightlight status | times         show what it's doing / today's fade times
nightlight preview [secs]         show full warmth briefly
nightlight detect                 find DDC/CI monitors
nightlight timer on | off         background schedule (systemd user timer)
```
