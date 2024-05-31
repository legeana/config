function fish_user_key_bindings
    bind \cd delete-char
    # PuTTY
    bind \eOD backward-word
    bind \eOC forward-word

    # Ctrl-Left and Ctrl-W
    bind \e\[1\;5D backward-word
    bind \cW backward-kill-word
end

# Useful resources:
# - https://blog.sanctum.geek.nz/putty-configuration/
#   - TERM=putty-256color comes from ncurses-term package
#   - Set Connection > Data >> Terminal-type string to putty-256color.
#   - Window > Colours >> Allow terminal to use xterm 256-colour mode.
#   - Window > Translation >> Remote character set = UTF-8.
#   - Window > Appearance >> Font = Consolas.
#   - Terminal > Bell = None.
#   - Window > Selection >> Action of mouse buttons: xterm
# - https://github.com/jblaine/solarized-and-modern-putty/blob/master/putty-modern-256color.reg
# - https://puttytray.goeswhere.com/
