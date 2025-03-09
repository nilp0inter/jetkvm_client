-- This script simulates an Alt+Tab sequence on Windows.
-- Step 1: Hold down the Alt key.
--   - The combo uses a modifier value of 0x04 (representing Alt) and sends key 0xE2 (the Alt key code).
--   - Both hold_modifiers and hold_keys are enabled so that the Alt key remains pressed.
--   - After sending, it waits 100ms before processing the next combo.
--
-- Step 2: Press the Tab key while keeping Alt held.
--   - The combo still uses the Alt modifier (0x04) and sends key 0x2B (Tab).
--   - The key is held for 50ms and then the script waits 3000ms.
--
-- Step 3: Simulate repeated Tab presses while Alt remains held.
--   - A sequence of combos sends Tab (0x2B) with a hold period (e.g., 200ms) and then sends an empty key press with a wait period (e.g., 2000ms).
--
-- Step 4: Press Esc to cancel the switcher and then release Alt.
--   - First, a combo sends Esc (0x29) with an instant_release flag.
--   - Finally, a combo with modifier 0x04 sends Alt (0xE2) with instant_release to clear the Alt hold.
--
-- The expectation is that during the entire sequence, Alt (0xE2) remains held until we explicitly release it.
-- The log should show active_modifiers = 0x04 while Alt is held and active_keys containing {226}.
-- Finally, after the instant releases, both active_modifiers and active_keys should be cleared.


-- send_key_combinations parameters documentation:
-- ----------------------------------------------
-- send_key_combinations accepts a table (list) of key combo elements.
-- Each element in the table represents a keystroke (or a set of keystrokes) and includes parameters
-- that define how that keystroke is simulated. Here's a breakdown of each parameter:
-- 
--
-- modifier:
--   - A bitmask representing modifier keys (e.g., 0x04 for Alt).
--   - This value is combined with the keys provided to simulate simultaneous key presses.
--
-- keys:
--   - A table of keycodes (in hexadecimal) to be sent.
--   - Example: { 0xE2 } for the Alt key or { 0x2B } for the Tab key.
--   - If omitted (nil), it can default to an empty table, which is useful for pure waits.
--
-- hold_keys:
--   - A boolean flag.
--   - When true, the keys in this combo are added to the active_keys state and remain pressed until explicitly released.
--   - This is used when you want to keep a key (like Alt) pressed across multiple combos.
--
-- hold_modifiers:
--   - A boolean flag.
--   - When true, the modifier bits (e.g., 0x04 for Alt) are added to the active_modifiers state and are not cleared automatically.
--   - This ensures that even if subsequent combos release keys, the modifier (Alt) stays active.
--
-- hold:
--   - An optional number (milliseconds) that specifies how long to hold the key press before releasing it.
--   - During this period, the keys (and possibly modifiers) remain active.
--
-- wait:
--   - An optional number (milliseconds) to wait after processing the current combo before proceeding to the next combo.
--   - Useful for adding delays between key sequences (e.g., waiting for the switcher UI to appear).
--
-- instant_release:
--   - A boolean flag.
--   - When true, the specified keys (and possibly modifiers) are immediately released after being processed.
--   - The release logic can clear the keys/modifiers unless they are being held by hold_keys/hold_modifiers.
--
-- clear_keys:
--   - An optional boolean flag.
--   - When true, it forces an immediate reset by clearing all active keys and modifiers.
--
--
--
--
--
-- How and why the Alt key remains pressed:
--   - In the first combo, both hold_keys and hold_modifiers are set to true.
--   - The Alt key is sent using its keycode (0xE2) along with the Alt modifier (0x04).
--   - hold_keys ensures that the keycode 0xE2 is retained in the active_keys set.
--   - hold_modifiers ensures that the modifier 0x04 is retained in the active_modifiers set.
--   - This persistent state keeps Alt pressed across subsequent combos until a later combo explicitly releases it.



-- Example script simulating Alt+Tab with detailed timing and release:
send_key_combinations({
    -- Step 1: Hold Alt (0xE2)
    --   - Send Alt key with modifier 0x04.
    --   - Both hold_modifiers and hold_keys are enabled so that Alt remains pressed.
    { modifier = 0x04, keys = { 0xE2 }, wait = 100, hold_modifiers = true, hold_keys = true },

    -- Step 2: Press Tab (0x2B) while keeping Alt held.
    --   - Sends Tab while Alt remains active.
    --   - The Tab key is held for 50ms and then released, leaving Alt still pressed.
    { modifier = 0x04, keys = { 0x2B }, hold = 50, wait = 3000, hold_modifiers = true },

    -- Step 3: Additional Tab presses to simulate switching windows.
    { modifier = 0x04, keys = { 0x2B }, hold = 200 },
    { modifier = 0x04, wait = 2000 }, -- pure wait
    { modifier = 0x04, keys = { 0x2B }, hold = 200 },
    { modifier = 0x04, wait = 2000 },

    -- Step 4: Press Esc (0x29) to cancel the switcher.
    { modifier = 0x00, keys = { 0x29 }, instant_release = true },

    -- Step 5: Release Alt.
    --   - Sends the Alt key with modifier 0x04 and instant_release to clear the held Alt state.
    { modifier = 0x04, keys = { 0xE2 }, instant_release = true },
})
