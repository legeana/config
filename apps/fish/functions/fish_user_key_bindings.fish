# fish_key_reader can be used to generate bind commands

function fish_user_key_bindings
    bind \cd delete-char
    # PuTTY
    bind \eOD backward-word
    bind \eOC forward-word
    # PuTTY + tmux
    bind \e\[D backward-word
    bind \e\[C forward-word
end
