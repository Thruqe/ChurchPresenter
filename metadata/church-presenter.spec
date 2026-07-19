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
mkdir -p $RPM_BUILD_ROOT/usr/share/icons/hicolor/512x512/apps

cp %{_sourcedir}/church-presenter $RPM_BUILD_ROOT/usr/bin/
cp %{_sourcedir}/libndi.so.4 $RPM_BUILD_ROOT/usr/lib64/
cp %{_sourcedir}/church-presenter.png $RPM_BUILD_ROOT/usr/share/icons/hicolor/512x512/apps/
cp %{_sourcedir}/church-presenter.desktop $RPM_BUILD_ROOT/usr/share/applications/

%files
/usr/bin/church-presenter
/usr/lib64/libndi.so.4
/usr/share/icons/hicolor/512x512/apps/church-presenter.png
/usr/share/applications/church-presenter.desktop

%changelog
* Sun Jul 19 2026 Daniel Peter <danielpeter0039@gmail.com> - 2.0.0-1
- Initial release with bundled libndi.so.4 and GTK4 assets
