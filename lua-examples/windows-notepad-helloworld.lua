print("Executing Lua script...")
send_windows_key()
delay(550)
send_text("notepad")
send_return()
delay(250)
delay(250)
send_text("Hello World!")
send_ctrl_a()
send_ctrl_c()
delay(250)
send_ctrl_v()
delay(250)
send_ctrl_v()
send_key_combinations({
    { modifier = 0x04, keys = {0x3D}, hold = 100, wait = 1000 }, -- Alt+F4
    { modifier = 0, keys = {0x4F}, wait = 250 }, -- Right arrow
    { modifier = 0, keys = {0x28} },  -- Return (Enter)
})
