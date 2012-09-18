export MPD_HOST="lex-pc" MPD_PORT="6600"
export EDITOR="vim" PAGER="most"
export LC_NUMERIC="C"

#case "$TERM" in
#   *xterm*|rxvt|(dt|k|E|a)term*) TERMTYPE="x";;
#   *) TERMTYPE="console";;
#esac

if [[ -n $DISPLAY ]]
then
    TERMTYPE="x"
else
    TERMTYPE="console"
fi

if [[ -f ~/.zshaliases ]]
then
    source ~/.zshaliases
fi

alias mplayer="mplayer -profile $TERMTYPE"

if [[ -f ~/.zshfunctions ]]
then
    source ~/.zshfunctions
fi
