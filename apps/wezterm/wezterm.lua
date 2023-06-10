local wezterm = require 'wezterm'
local config = {}

config.font = wezterm.font_with_fallback {
    'Hack Nerd Font',
    'DejaVu Sans Mono',
    'Consolas',
}

-- TODO: GNOME theme
config.color_scheme = 'Mathias'

-- Enable extended keys.
config.enable_kitty_keyboard = true
-- allow_win32_input_mode takes precedence, so must be disabled.
config.allow_win32_input_mode = false

return config
