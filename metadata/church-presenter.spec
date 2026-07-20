Name:           church-presenter
Version:        %{version}
Release:        1
Summary:        A lightweight, high-performance church presentation software
License:        MIT
URL:            https://github.com/thruqe/Church-Presenter

# Avoid building debuginfo packages
%define debug_package %{nil}

%description
Church Presenter is a lightweight, high-performance presentation software built with Rust and GTK4.
It allows church media teams to present Scripture verses and song lyrics on local displays as well as broadcast them as NDI streams.

%prep
# Nothing to do

%build
# Nothing to do

%install
rm -rf $RPM_BUILD_ROOT
mkdir -p $RPM_BUILD_ROOT/usr/bin
mkdir -p $RPM_BUILD_ROOT/usr/lib64
mkdir -p $RPM_BUILD_ROOT/usr/share/applications
mkdir -p $RPM_BUILD_ROOT/usr/share/pixmaps

# Icon directories for all required sizes
for SIZE in 16 24 32 48 64 128 256 512; do
    mkdir -p $RPM_BUILD_ROOT/usr/share/icons/hicolor/${SIZE}x${SIZE}/apps
done

cp %{_sourcedir}/church-presenter $RPM_BUILD_ROOT/usr/bin/
cp %{_sourcedir}/libndi.so.4 $RPM_BUILD_ROOT/usr/lib64/
mkdir -p $RPM_BUILD_ROOT/usr/share/church-presenter
cp %{_sourcedir}/KJV.sqlite $RPM_BUILD_ROOT/usr/share/church-presenter/ 2>/dev/null || true

# Install each pre-generated icon size
for SIZE in 16 24 32 48 64 128 256 512; do
    cp %{_sourcedir}/icons/church-presenter_${SIZE}.png \
       $RPM_BUILD_ROOT/usr/share/icons/hicolor/${SIZE}x${SIZE}/apps/church-presenter.png
done

# Legacy pixmaps (48px)
cp %{_sourcedir}/icons/church-presenter_48.png \
   $RPM_BUILD_ROOT/usr/share/pixmaps/church-presenter.png

cp %{_sourcedir}/church-presenter.desktop $RPM_BUILD_ROOT/usr/share/applications/

%post
gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
update-desktop-database /usr/share/applications 2>/dev/null || true

%postun
gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
update-desktop-database /usr/share/applications 2>/dev/null || true

%files
/usr/bin/church-presenter
/usr/lib64/libndi.so.4
/usr/share/applications/church-presenter.desktop
/usr/share/pixmaps/church-presenter.png
/usr/share/icons/hicolor/16x16/apps/church-presenter.png
/usr/share/icons/hicolor/24x24/apps/church-presenter.png
/usr/share/icons/hicolor/32x32/apps/church-presenter.png
/usr/share/icons/hicolor/48x48/apps/church-presenter.png
/usr/share/icons/hicolor/64x64/apps/church-presenter.png
/usr/share/icons/hicolor/128x128/apps/church-presenter.png
/usr/share/icons/hicolor/256x256/apps/church-presenter.png
/usr/share/icons/hicolor/512x512/apps/church-presenter.png

%changelog
* Mon Jul 20 2026 Daniel Peter <danielpeter0039@gmail.com> - 3.0.0-1
- Bundle KJV.sqlite database across Windows, Linux, and macOS installers
- Implement multi-fallback user data directory handling for read-only install paths
- Automatic recursive BFS dependency resolution for Windows DLL packaging
