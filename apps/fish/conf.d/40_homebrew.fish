must-have-command brew

eval (brew shellenv)
set _list "$HOMEBREW_PREFIX/opt/"*/libexec/gnubin; add_to_path $_list
set _list "$HOMEBREW_PREFIX/opt/"*/libexec/gnuman; add_to_manpath $_list
