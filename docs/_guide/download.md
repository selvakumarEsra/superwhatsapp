---
title: Download
description: Download SuperWhatsapp for macOS, Windows or Linux, free during the beta, with install steps for each.
nav_order: 2
---

{% assign v = site.superwhatsapp_version %}
{% assign repo = site.release_repository %}
{% assign name = site.release_asset_prefix %}
{% assign app = site.release_app_name %}
{% assign base = "https://github.com/" | append: repo | append: "/releases/download/v" | append: v %}

The current version is **{{ app }} {{ v }}**, free during the beta. Every
download is listed with its SHA-256 checksum in
[checksums.txt]({{ base }}/checksums.txt), signed with the project's update
key. Older versions are on the
[releases page](https://github.com/{{ repo }}/releases).

<div class="sw-downloads">
  <a href="{{ base }}/{{ name }}-v{{ v }}-macos-universal.dmg"><strong>macOS</strong><span>Apple silicon and Intel, .dmg</span></a>
  <a href="{{ base }}/{{ name }}-v{{ v }}-x86_64-pc-windows-msvc.msi"><strong>Windows</strong><span>x64 installer, .msi</span></a>
  <a href="{{ base }}/{{ name }}-v{{ v }}-x86_64-unknown-linux-gnu.tar.gz"><strong>Linux</strong><span>x86_64, .tar.gz</span></a>
</div>

After installing, [link your phone](/getting-started/#link-with-your-phone)
and [turn on Znga](/znga/#set-up-znga) to get the AI agent.

## macOS

One download for both Apple silicon and Intel Macs, macOS 11 or newer:

- [{{ name }}-v{{ v }}-macos-universal.dmg]({{ base }}/{{ name }}-v{{ v }}-macos-universal.dmg)

Open it and drag **{{ app }}** to Applications. The beta is not yet signed
with an Apple Developer ID, so the first time, right-click {{ app }} in
Applications and choose **Open**, then **Open** again.

## Windows

The installer is a Windows Installer package. It adds {{ app }} to the Start
menu and needs no administrator rights. Choose x64 for most PCs and arm64 for
Windows on ARM:

- [{{ name }}-v{{ v }}-x86_64-pc-windows-msvc.msi]({{ base }}/{{ name }}-v{{ v }}-x86_64-pc-windows-msvc.msi)
- [{{ name }}-v{{ v }}-aarch64-pc-windows-msvc.msi]({{ base }}/{{ name }}-v{{ v }}-aarch64-pc-windows-msvc.msi)

To run {{ app }} without installing it, extract a zip and run
`{{ name }}.exe`:

- [{{ name }}-v{{ v }}-x86_64-pc-windows-msvc.zip]({{ base }}/{{ name }}-v{{ v }}-x86_64-pc-windows-msvc.zip)
- [{{ name }}-v{{ v }}-aarch64-pc-windows-msvc.zip]({{ base }}/{{ name }}-v{{ v }}-aarch64-pc-windows-msvc.zip)

SmartScreen may warn about an unknown publisher on first run, because the
beta is not code-signed yet. Choose **More info**, then **Run anyway**.
{{ app }} draws with OpenGL and needs a graphics driver with OpenGL 2.1 or
newer.

## Linux

On any distribution with [Flatpak](https://flatpak.org), install the bundle.
It runs in Flatpak's sandbox and reaches your files only through the system's
file picker:

```sh
flatpak install --user ./{{ name }}-v{{ v }}-x86_64.flatpak
```

- [{{ name }}-v{{ v }}-x86_64.flatpak]({{ base }}/{{ name }}-v{{ v }}-x86_64.flatpak)

Or take a tarball with the program, its desktop file and icon, and install it
for your user with the script inside:

```sh
tar xzf {{ name }}-v{{ v }}-x86_64-unknown-linux-gnu.tar.gz
cd {{ name }}-v{{ v }}-x86_64-unknown-linux-gnu
packaging/install-user.sh ./{{ name }}
```

- [{{ name }}-v{{ v }}-x86_64-unknown-linux-gnu.tar.gz]({{ base }}/{{ name }}-v{{ v }}-x86_64-unknown-linux-gnu.tar.gz)
- [{{ name }}-v{{ v }}-aarch64-unknown-linux-gnu.tar.gz]({{ base }}/{{ name }}-v{{ v }}-aarch64-unknown-linux-gnu.tar.gz)

{{ app }} needs glibc 2.36 or newer (Debian 12, Ubuntu 22.10, Fedora 37 and
newer), ALSA and libpipewire 0.3.53 or newer, plus the usual graphics
libraries: `libglvnd`, `libxkbcommon`, `wayland` and `libx11`. On Debian or
Ubuntu: `libasound2t64`, `libpipewire-0.3-0t64`, `libgl1`, `libxkbcommon0`
and `libwayland-client0`.

With [Nix](https://nixos.org), install from the flake:

```sh
nix profile install github:{{ repo }}
```

## Updates

{{ app }} checks this project's releases and tells you when a new version is
out. Installed copies update themselves after verifying the release's
signature; a copy that cannot (the Flatpak, for example) shows where to
download the new version.

## Coming from ZapFast?

{{ app }} is built on ZapFast and uses the same folders, so installing it
keeps your linked numbers, chats, archive and settings. Quit ZapFast first:
only one of the two runs at a time.

## Try the new interface

Each release also carries **{{ app }} Preview**, a new interface modelled on
the desktop WhatsApp you know. It installs beside {{ app }}, opens the same
account, and is still missing some features. Look for the
`{{ name }}-preview-*` files on the
[release page](https://github.com/{{ repo }}/releases/tag/v{{ v }}).

<style>
  .sw-downloads { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin: 24px 0; }
  .sw-downloads a { display: flex; flex-direction: column; gap: 4px; padding: 16px 18px; border-radius: 12px; background: var(--vp-c-brand-soft); text-decoration: none !important; }
  .sw-downloads strong { color: var(--vp-c-brand-1); font-size: 17px; }
  .sw-downloads span { color: var(--vp-c-text-2); font-size: 13px; }
</style>
