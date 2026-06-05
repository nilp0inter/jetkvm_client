{
  description = "A Rust project with a development shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        runtimeLibs = with pkgs; [
          openssl.out
        ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
          vulkan-loader
          libxkbcommon
          libGL
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
            openssl
            pkg-config
            gst_all_1.gstreamer
            gst_all_1.gst-plugins-base
            gst_all_1.gst-plugins-good
            gst_all_1.gst-plugins-bad
            gst_all_1.gst-libav
            glib
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            vulkan-loader
            vulkan-headers
            vulkan-tools
            libxkbcommon
            libGL
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            libiconv
            apple-sdk_14
          ];
          shellHook = ''
            ${pkgs.lib.optionalString pkgs.stdenv.isLinux "export LD_LIBRARY_PATH=\"${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH\""}
            export GST_PLUGIN_SYSTEM_PATH_1_0="${pkgs.gst_all_1.gstreamer.out}/lib/gstreamer-1.0:${pkgs.gst_all_1.gst-plugins-base}/lib/gstreamer-1.0:${pkgs.gst_all_1.gst-plugins-good}/lib/gstreamer-1.0:${pkgs.gst_all_1.gst-plugins-bad}/lib/gstreamer-1.0:${pkgs.gst_all_1.gst-libav}/lib/gstreamer-1.0"
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.glib.dev}/lib/pkgconfig:${pkgs.gst_all_1.gstreamer.dev}/lib/pkgconfig:${pkgs.gst_all_1.gst-plugins-base.dev}/lib/pkgconfig"
            
            ${pkgs.lib.optionalString pkgs.stdenv.isDarwin ''
              # Helper to sign binaries and create an app bundle on macOS
              # This is required to trigger the "Local Network" permission prompt.
              sign_jetkvm() {
                ENT="entitlements.plist"
                if [ ! -f "$ENT" ]; then
                  echo "Creating $ENT..."
                  cat <<EOF > "$ENT"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>
</dict>
</plist>
EOF
                fi

                if [ -f "target/debug/jetkvm_viewer" ]; then
                  echo "Creating JetKVM.app bundle..."
                  mkdir -p JetKVM.app/Contents/MacOS
                  cp target/debug/jetkvm_viewer JetKVM.app/Contents/MacOS/JetKVM
                  
                  cat <<EOF > JetKVM.app/Contents/Info.plist
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>JetKVM</string>
    <key>CFBundleIdentifier</key>
    <string>com.jetkvm.viewer</string>
    <key>CFBundleName</key>
    <string>JetKVM</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>NSLocalNetworkUsageDescription</key>
    <string>JetKVM needs local network access to connect to your JetKVM device.</string>
</dict>
</plist>
EOF
                  echo "Signing JetKVM.app bundle..."
                  codesign --force --deep --entitlements "$ENT" -s - JetKVM.app
                fi

                if [ -f "target/debug/jetkvm_client" ]; then
                  echo "Signing target/debug/jetkvm_client..."
                  codesign --force --deep --entitlements "$ENT" -s - target/debug/jetkvm_client
                fi
              }
              export -f sign_jetkvm
              echo "macOS detected: Use 'sign_jetkvm' to sign binaries and create JetKVM.app to fix Local Network permissions."
              echo "To trigger permissions, run: open JetKVM.app --args -H <HOST> -P <PWD>"
            ''}
          '';
        };
      }
    );
}
