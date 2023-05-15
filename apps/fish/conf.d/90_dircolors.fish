must-have-command dircolors

eval (dircolors --csh)

# dircolors doesn't always support alacritty
if test "$TERM" = alacritty -a -z "$LS_COLORS"
    eval (env TERM=xterm-256color dircolors --csh)
end
