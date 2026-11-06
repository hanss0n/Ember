# Night Light for COSMIC

Your screen gets warmer in the evening so it's easier on the eyes, then goes back to normal in the morning. This is the same idea as Night Shift on macOS or Night Light on GNOME, built for the COSMIC desktop.

<p align="center">
  <img src="docs/settings.png" alt="The Night Light settings window" width="560">
</p>

## Why this exists

COSMIC doesn't have a night light yet. The usual Linux tools (Redshift, gammastep, wlsunset) don't work on it either, because COSMIC doesn't let other apps change the screen's colors.

This project gets around that by talking to the **monitors themselves**. Most external monitors accept color commands over the display cable, a standard called DDC/CI. Night Light uses that to lower each monitor's blue and green levels at night, and then restores its normal color preset in the morning.

## What you get

**A settings app.** Open "Night Light" from your app launcher to change:

- **Mode**: follow a schedule, stay warm, or stay off.
- **Schedule**: when the screen starts warming up in the evening, when it cools down in the morning, and how long the fade takes. The change is gradual, so you shouldn't notice it happening.
- **Warmth**: how warm the screen gets. When you let go of the slider it shows you full warmth for a few seconds.
- **Monitors**: which screens to control.
- **Run automatically**: keep the schedule running in the background, even when the window is closed.

**A panel icon.** A moon while the screen is warm, a sun when it isn't. Click it to switch modes or open the settings.

<p align="center">
  <img src="docs/panel.png" alt="The moon icon in the COSMIC panel" height="60">
</p>

**A command-line tool** for scripts and quick changes:

```
nightlight night       # warm now, until you switch back
nightlight day         # normal colors, until you switch back
nightlight auto        # follow the schedule again
nightlight status      # what it's doing right now
nightlight times       # today's fade times
```

Times always follow your computer's clock, so daylight saving time is handled automatically.

## Will it work for me?

You need:

- The **COSMIC** desktop (made for Pop!_OS 24.04, but any COSMIC setup should work).
- **External monitors with DDC/CI** turned on. Most monitors support it, but some ship with it off in the on-screen menu. Laptop screens don't support it.
- Monitors with a **"User" color mode** whose red/green/blue levels can be adjusted. Most monitors have one.

To check your monitors, install `ddcutil` and run `ddcutil detect`. Each monitor it lists should work.

## Installing

1. Install the tools Night Light needs, and give yourself access to the monitor connection:

   ```
   sudo apt install ddcutil just
   sudo usermod -aG i2c $USER
   ```

   Then log out and back in so the group change takes effect.

2. Build and install. Everything goes into your home folder, so no `sudo` is needed:

   ```
   just install
   ```

   Building needs the Rust toolchain ([rustup.rs](https://rustup.rs)).

3. Open **Night Light** from the app launcher, click **Detect** under Monitors, and turn on **Run automatically**.

4. To add the panel icon, open **Settings → Desktop → Panel → Configure panel applets**, click **Add applet**, and pick **Night Light**. If it isn't in the list, log out and back in.

## Uninstalling

First remove the panel icon in **Settings → Desktop → Panel**. Then run:

```
just uninstall
```

This stops the background schedule, puts your monitors back to normal colors, and removes the installed files. Your settings stay in `~/.config/nightlight/`, so delete that folder too if you want everything gone.

If a monitor stays warm afterwards, pick its standard or 6500 K color preset in its on-screen menu.

## Good to know

- **Each monitor's own menu will show a "User" color mode at night.** That's Night Light at work. Switching presets in the monitor menu will override it until the next change.
- **Changes aren't instant.** Talking to monitors over the cable takes about a second per monitor. Fades happen in small steps a few minutes apart.
- **Some monitors misreport their state.** For example, the MSI MAG 341C OLED reports the wrong color preset back, but it does apply the change. If a monitor ignores the change, try `nightlight night` again, or untick it under Monitors.

## For developers

The project has three parts that share one config file (`~/.config/nightlight/config.json`):

| Part | Written in | Role |
|---|---|---|
| `cli/nightlight` | Python | Does the actual monitor control (via `ddcutil`), the schedule math, and the background timer. |
| `cosmic-nightlight` | Rust, [libcosmic](https://github.com/pop-os/libcosmic) | The settings app. |
| `cosmic-applet-nightlight` | Rust, libcosmic | The panel icon and popup. |

The GUIs never talk to the monitors directly. They save settings and ask the CLI to apply them, so there's only one place that changes the screen.

```
src/
  config.rs        settings: load, save, defaults
  schedule.rs      "how warm should it be at 21:30?" (with tests)
  backend.rs       runs the CLI and checks the background timer
  status.rs        status text and sun/moon icon
  paths.rs         where files live
  bin/app/         settings window
  bin/applet/      panel icon and popup
cli/nightlight     command-line tool
res/               app launcher entries
```

Useful commands:

```
just test          # run the tests
just check         # lint with clippy
just install       # build and install for your user
```
