# SuperWhatsapp built from its release source, for Fedora COPR and
# openSUSE OBS. Cargo fetches the locked dependencies while building, so the
# COPR project and the OBS package need network access during builds.

Name:           superwhatsapp
Version:        @VERSION@
Release:        1%{?dist}
Summary:        Fast native WhatsApp client
License:        MIT
URL:            https://selvakumaresra.github.io/superwhatsapp
Source0:        https://github.com/selvakumarEsra/superwhatsapp/archive/refs/tags/v%{version}.tar.gz#/%{name}-%{version}.tar.gz

# ring compiles its own C, and LTO objects in its archive leave the link
# without ring_core_* symbols, as on Arch.
%global _lto_cflags %{nil}
# The binary is stripped of its source paths below; debuginfo would only
# carry them back.
%global debug_package %{nil}

ExclusiveArch:  x86_64 aarch64
BuildRequires:  rust >= 1.98
BuildRequires:  cargo
BuildRequires:  cmake
BuildRequires:  clang
BuildRequires:  clang-devel
BuildRequires:  gcc-c++
BuildRequires:  perl
BuildRequires:  pkgconfig(alsa)
BuildRequires:  pkgconfig(libpipewire-0.3)
Requires:       libglvnd-glx
Requires:       libglvnd-egl
Requires:       libxkbcommon
Requires:       libxkbcommon-x11
Requires:       libwayland-client
Requires:       libwayland-cursor
Requires:       libwayland-egl
Requires:       libX11
Requires:       libX11-xcb
Requires:       libXcursor
Requires:       libXi
Requires:       libXrandr
Requires:       alsa-lib
Requires:       pipewire-libs
Recommends:     google-noto-color-emoji-fonts
Recommends:     xdg-desktop-portal
Provides:       fastsapp = %{version}-%{release}
Obsoletes:      fastsapp < %{version}-%{release}

%description
SuperWhatsapp is a native WhatsApp client for the desktop, written in Rust with
egui: no browser engine, about 200 MB of memory when idle. It links as a
companion device of your phone and keeps your chats in an encrypted local
archive.

%prep
%autosetup -n %{name}-%{version}

%build
export CARGO_TARGET_DIR=target
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=%{_builddir}=/"
cargo build --locked --release

%install
install -Dm755 target/release/superwhatsapp %{buildroot}%{_bindir}/superwhatsapp
install -Dm644 packaging/applications/superwhatsapp.desktop %{buildroot}%{_datadir}/applications/superwhatsapp.desktop
install -Dm644 packaging/icons/superwhatsapp.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/superwhatsapp.svg
install -Dm644 contrib/omarchy/superwhatsapp.json.tpl %{buildroot}%{_datadir}/superwhatsapp/omarchy/superwhatsapp.json.tpl
install -Dm755 contrib/omarchy/superwhatsapp-theme %{buildroot}%{_datadir}/superwhatsapp/omarchy/superwhatsapp-theme

%files
%license LICENSE THIRD-PARTY-NOTICES.md
%doc README.md
%{_bindir}/superwhatsapp
%{_datadir}/applications/superwhatsapp.desktop
%{_datadir}/icons/hicolor/scalable/apps/superwhatsapp.svg
%{_datadir}/superwhatsapp/

%changelog
* @DATE@ Selvakumar Esra <selvakumar.esra.ai@gmail.com> - @VERSION@-1
- SuperWhatsapp @VERSION@: https://github.com/selvakumarEsra/superwhatsapp/releases/tag/v@VERSION@
