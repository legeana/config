-- {{header("--")}}

local wezterm = require 'wezterm'
local config = {}

config.font = wezterm.font_with_fallback {
    'Hack Nerd Font',
    'DejaVu Sans Mono',
    'Consolas',
}

-- TODO: GNOME theme
config.color_scheme = 'Mathias'

{% if is_windows() -%}
config.default_prog = { 'powershell' }
{%- endif %}

-- Enable extended keys.
config.enable_kitty_keyboard = true
-- allow_win32_input_mode takes precedence, so must be disabled.
config.allow_win32_input_mode = false

-- Often tabs aren't used.
config.hide_tab_bar_if_only_one_tab = true

-- Remove unnecessary padding.
config.window_padding = {
    left = 0,
    right = 0,
    top = 0,
    bottom = 0,
}

config.hyperlink_rules = wezterm.default_hyperlink_rules()
-- Markdown links.
table.insert(config.hyperlink_rules, {
    regex = '\\[[^]]+\\]\\(([^)]+)\\)',
    format = '$1',
})

return config
