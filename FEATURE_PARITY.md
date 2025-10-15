# JetKVM Client Feature Parity Analysis

This document compares the remote command functionality between the original Web UI and the Rust implementation.

**Legend:**
- ✅ Fully Implemented (Library + CLI)
- 🔶 Partially Implemented (Library only, no CLI)
- ❌ Not Implemented

---

## HID (Human Interface Device) Commands

### Keyboard

| Method | UI | Rust Library | CLI | Status | Notes |
|--------|----|--------------|----|--------|-------|
| `keyboardReport` | ✅ | ✅ `rpc_keyboard_report()` | ✅ `keyboard-report` | ✅ | Basic keyboard HID report |
| `getKeyboardLayout` | ✅ | ❌ | ❌ | ❌ | Get current keyboard layout |
| `setKeyboardLayout` | ✅ | ❌ | ❌ | ❌ | Set keyboard layout |
| `getKeyboardLedState` | ✅ | ❌ | ❌ | ❌ | Get LED state (Caps/Num Lock) |
| `getKeyDownState` | ✅ | ❌ | ❌ | ❌ | Get currently pressed keys |

**High-level keyboard helpers:**
| Function | UI | Rust Library | CLI | Status |
|----------|----|--------------|----|--------|
| Send text (ASCII) | ✅ | ✅ `rpc_sendtext()` | ✅ `sendtext` | ✅ |
| Send text with layout | ✅ | ✅ `send_text_with_layout()` | ✅ `send-text-with-layout` | ✅ |
| Send Return/Enter | ✅ | ✅ `send_return()` | ✅ `send-return` | ✅ |
| Send Ctrl-C | ✅ | ✅ `send_ctrl_c()` | ✅ `send-ctrl-c` | ✅ |
| Send Ctrl-V | ✅ | ✅ `send_ctrl_v()` | ✅ `send-ctrl-v` | ✅ |
| Send Ctrl-X | ✅ | ✅ `send_ctrl_x()` | ✅ `send-ctrl-x` | ✅ |
| Send Ctrl-A | ✅ | ✅ `send_ctrl_a()` | ✅ `send-ctrl-a` | ✅ |
| Send Windows key | ✅ | ✅ `send_windows_key()` | ✅ `send-windows-key` | ✅ |
| Key combinations | ✅ | ✅ `send_key_combinations()` | ❌ | 🔶 |

### Mouse

| Method | UI | Rust Library | CLI | Status | Notes |
|--------|----|--------------|----|--------|-------|
| `absMouseReport` | ✅ | ✅ `rpc_abs_mouse_report()` | ✅ `abs-mouse-report` | ✅ | Absolute mouse positioning |
| `relMouseReport` | ✅ | ❌ | ❌ | ❌ | Relative mouse movement |
| `wheelReport` | ✅ | ✅ `rpc_wheel_report()` | ✅ `wheel-report` | ✅ | Mouse wheel scrolling |

**High-level mouse helpers:**
| Function | UI | Rust Library | CLI | Status |
|----------|----|--------------|----|--------|
| Move mouse | ✅ | ✅ `rpc_move_mouse()` | ✅ `move-mouse` | ✅ |
| Left click | ✅ | ✅ `rpc_left_click()` | ✅ `left-click` | ✅ |
| Right click | ✅ | ✅ `rpc_right_click()` | ✅ `right-click` | ✅ |
| Middle click | ✅ | ✅ `rpc_middle_click()` | ✅ `middle-click` | ✅ |
| Double click | ✅ | ✅ `rpc_double_click()` | ✅ `double-click` | ✅ |
| Click and drag | ❌ | ✅ `rpc_left_click_and_drag_to_center()` | ❌ | 🔶 |

### Mouse Jiggler

| Method | UI | Rust Library | CLI | Status |
|--------|----|--------------|----|--------|
| `getJigglerState` | ✅ | ❌ | ❌ | ❌ |
| `setJigglerState` | ✅ | ❌ | ❌ | ❌ |
| `getJigglerConfig` | ✅ | ❌ | ❌ | ❌ |
| `setJigglerConfig` | ✅ | ❌ | ❌ | ❌ |

---

## Video Commands

| Method | UI | Rust Library | CLI | Status | Notes |
|--------|----|--------------|----|--------|-------|
| `getVideoState` | ✅ | ❌ | ❌ | ❌ | Get video stream state |
| `getStreamQualityFactor` | ✅ | ❌ | ❌ | ❌ | Get video quality factor |
| `getEDID` | ✅ | ✅ `rpc_get_edid()` | ✅ `get-edid` | ✅ | Get EDID data |
| `setEDID` | ✅ | ✅ `rpc_set_edid()` | ✅ `set-edid` | ✅ | Set EDID configuration |
| `getVideoLogStatus` | ✅ | ❌ | ❌ | ❌ | Get video logging status |
| Screenshot | ✅ | ✅ `VideoFrameCapture::capture_screenshot_png()` | ✅ `screenshot` | ✅ | Capture PNG screenshot |

---

## Storage / Virtual Media Commands

| Method | UI | Rust Library | CLI | Status | Notes |
|--------|----|--------------|----|--------|-------|
| `getVirtualMediaState` | ✅ | ❌ | ❌ | ❌ | Get mounted media state |
| `mountWithHTTP` | ✅ | ❌ | ❌ | ❌ | Mount image from HTTP URL |
| `mountWithStorage` | ✅ | ❌ | ❌ | ❌ | Mount image from storage |
| `unmountImage` | ✅ | ❌ | ❌ | ❌ | Unmount virtual media |
| `listStorageFiles` | ✅ | ❌ | ❌ | ❌ | List stored ISO/image files |
| `getStorageSpace` | ✅ | ❌ | ❌ | ❌ | Get available storage space |
| `deleteStorageFile` | ✅ | ❌ | ❌ | ❌ | Delete file from storage |
| `startStorageFileUpload` | ✅ | ❌ | ❌ | ❌ | Upload file to storage |

---

## Network Commands

| Method | UI | Rust Library | CLI | Status | Notes |
|--------|----|--------------|----|--------|-------|
| `getNetworkSettings` | ✅ | ❌ | ❌ | ❌ | Get network configuration |
| `setNetworkSettings` | ✅ | ❌ | ❌ | ❌ | Set network configuration |
| `getNetworkState` | ✅ | ❌ | ❌ | ❌ | Get current network state |
| `renewDHCPLease` | ✅ | ❌ | ❌ | ❌ | Renew DHCP lease |

### Wake-on-LAN

| Method | UI | Rust Library | CLI | Status |
|--------|----|--------------|----|--------|
| `getWakeOnLanDevices` | ✅ | ❌ | ❌ | ❌ |
| `setWakeOnLanDevices` | ✅ | ❌ | ❌ | ❌ |
| `sendWOLMagicPacket` | ✅ | ❌ | ❌ | ❌ |

---

## Power Control Commands

### ATX Power Control

| Method | UI | Rust Library | CLI | Status | Notes |
|--------|----|--------------|----|--------|-------|
| `getATXState` | ✅ | ❌ | ❌ | ❌ | Get ATX power state |
| `setATXPowerAction` | ✅ | ❌ | ❌ | ❌ | Power on/off/reset actions |

### DC Power Control

| Method | UI | Rust Library | CLI | Status |
|--------|----|--------------|----|--------|
| `getDCPowerState` | ✅ | ❌ | ❌ | ❌ |
| `setDCPowerState` | ✅ | ❌ | ❌ | ❌ |
| `setDCRestoreState` | ✅ | ❌ | ❌ | ❌ |

---

## USB Commands

| Method | UI | Rust Library | CLI | Status | Notes |
|--------|----|--------------|----|--------|-------|
| `getUsbConfig` | ✅ | ❌ | ❌ | ❌ | Get USB device configuration |
| `setUsbConfig` | ✅ | ❌ | ❌ | ❌ | Set USB device configuration |
| `getUsbDevices` | ✅ | ❌ | ❌ | ❌ | List USB devices |
| `setUsbDevices` | ✅ | ❌ | ❌ | ❌ | Configure USB devices |
| `getUsbEmulationState` | ✅ | ❌ | ❌ | ❌ | Get USB emulation state |
| `setUsbEmulationState` | ✅ | ❌ | ❌ | ❌ | Set USB emulation state |

---

## System / Device Commands

| Method | UI | Rust Library | CLI | Status | Notes |
|--------|----|--------------|----|--------|-------|
| `ping` | ✅ | ✅ `rpc_ping()` | ✅ `ping` | ✅ | Basic connectivity test |
| `getDeviceID` | ✅ | ✅ `rpc_get_device_id()` | ✅ `get-device-id` | ✅ | Get device identifier |
| `reboot` | ✅ | ❌ | ❌ | ❌ | Reboot the device |
| `getLocalVersion` | ✅ | ❌ | ❌ | ❌ | Get firmware version |
| `getUpdateStatus` | ✅ | ❌ | ❌ | ❌ | Get firmware update status |
| `tryUpdate` | ✅ | ❌ | ❌ | ❌ | Attempt firmware update |
| `getAutoUpdateState` | ✅ | ❌ | ❌ | ❌ | Get auto-update setting |
| `setAutoUpdateState` | ✅ | ❌ | ❌ | ❌ | Set auto-update setting |
| `getTimezones` | ✅ | ❌ | ❌ | ❌ | List available timezones |

---

## Hardware Settings Commands

| Method | UI | Rust Library | CLI | Status | Notes |
|--------|----|--------------|----|--------|-------|
| `setDisplayRotation` | ✅ | ❌ | ❌ | ❌ | Rotate display orientation |
| `getBacklightSettings` | ✅ | ❌ | ❌ | ❌ | Get backlight configuration |
| `setBacklightSettings` | ✅ | ❌ | ❌ | ❌ | Set backlight configuration |

---

## Cloud / Access Commands

| Method | UI | Rust Library | CLI | Status | Notes |
|--------|----|--------------|----|--------|-------|
| `getCloudState` | ✅ | ❌ | ❌ | ❌ | Get cloud connection state |
| `setCloudUrl` | ✅ | ❌ | ❌ | ❌ | Set cloud URL |
| `getTLSState` | ✅ | ❌ | ❌ | ❌ | Get TLS/SSL state |
| `setTLSState` | ✅ | ❌ | ❌ | ❌ | Set TLS/SSL state |
| `deregisterDevice` | ✅ | ❌ | ❌ | ❌ | Deregister from cloud |

---

## Extension Commands

| Method | UI | Rust Library | CLI | Status | Notes |
|--------|----|--------------|----|--------|-------|
| `getActiveExtension` | ✅ | ❌ | ❌ | ❌ | Get active extension ID |
| `setActiveExtension` | ✅ | ❌ | ❌ | ❌ | Set active extension |
| `getSerialSettings` | ✅ | ❌ | ❌ | ❌ | Get serial console settings |
| `setSerialSettings` | ✅ | ❌ | ❌ | ❌ | Set serial console settings |

---

## Advanced Settings Commands

| Method | UI | Rust Library | CLI | Status | Notes |
|--------|----|--------------|----|--------|-------|
| `getDevModeState` | ✅ | ❌ | ❌ | ❌ | Get developer mode state |
| `setDevModeState` | ✅ | ❌ | ❌ | ❌ | Set developer mode state |
| `getSSHKeyState` | ✅ | ❌ | ❌ | ❌ | Get SSH key configuration |
| `setSSHKeyState` | ✅ | ❌ | ❌ | ❌ | Set SSH key |
| `getDevChannelState` | ✅ | ❌ | ❌ | ❌ | Get dev channel state |
| `setDevChannelState` | ✅ | ❌ | ❌ | ❌ | Set dev channel state |
| `getLocalLoopbackOnly` | ✅ | ❌ | ❌ | ❌ | Get loopback-only setting |
| `setLocalLoopbackOnly` | ✅ | ❌ | ❌ | ❌ | Set loopback-only setting |
| `getUsbEmulationState` | ✅ | ❌ | ❌ | ❌ | Get USB emulation state |
| `setUsbEmulationState` | ✅ | ❌ | ❌ | ❌ | Set USB emulation state |
| `resetConfig` | ✅ | ❌ | ❌ | ❌ | Reset to factory defaults |

---

## Utility Commands (CLI only)

| Function | UI | Rust Library | CLI | Status |
|----------|----|--------------|----|--------|
| Wait/Sleep | ❌ | N/A | ✅ `wait` | ✅ |

---

## Summary Statistics

### Overall Implementation Status

| Category | Total Methods | Fully Implemented | Partially Implemented | Not Implemented |
|----------|---------------|-------------------|-----------------------|-----------------|
| **HID - Keyboard** | 5 base + 9 helpers | 8 | 1 | 5 |
| **HID - Mouse** | 3 base + 6 helpers | 8 | 1 | 1 |
| **Mouse Jiggler** | 4 | 0 | 0 | 4 |
| **Video** | 6 | 3 | 0 | 3 |
| **Storage/Virtual Media** | 8 | 0 | 0 | 8 |
| **Network** | 7 | 0 | 0 | 7 |
| **Power Control** | 5 | 0 | 0 | 5 |
| **USB** | 6 | 0 | 0 | 6 |
| **System/Device** | 9 | 2 | 0 | 7 |
| **Hardware Settings** | 3 | 0 | 0 | 3 |
| **Cloud/Access** | 5 | 0 | 0 | 5 |
| **Extensions** | 4 | 0 | 0 | 4 |
| **Advanced Settings** | 10 | 0 | 0 | 10 |
| **TOTAL** | **75** | **21** | **2** | **52** |

### Completion Percentage
- **Fully Implemented**: 21/75 = **28%**
- **Partially Implemented**: 2/75 = **2.7%**
- **Not Implemented**: 52/75 = **69.3%**

---

## Priority Recommendations

### High Priority (Core Functionality)
1. **Virtual Media/Storage** - Critical for OS installation and recovery
2. **Network Configuration** - Essential for device management
3. **Power Control (ATX)** - Core KVM functionality
4. **System Commands** (reboot, firmware updates)

### Medium Priority (Enhanced Features)
1. **Keyboard Layout Management** - Better international support
2. **Mouse Jiggler** - Convenience feature
3. **USB Configuration** - Device management
4. **Video Settings** - Quality control

### Low Priority (Advanced/Admin)
1. **Cloud/Access Settings** - Administrative
2. **Advanced Settings** - Power user features
3. **Extensions** - Optional functionality
4. **Hardware Settings** - Device-specific
