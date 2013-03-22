export MPD_HOST="lex-pc" MPD_PORT="6600"
export EDITOR="vim" PAGER="most"
export LC_NUMERIC="C"

#case "$TERM" in
#   *xterm*|rxvt|(dt|k|E|a)term*) MPLAYER_PROFILE="x";;
#   *) MPLAYER_PROFILE="console";;
#esac

if [[ -n $DISPLAY ]]
then
    MPLAYER_PROFILE="x"
elif [[ -n $TMUX || -n $SSH_CLIENT || $TERM = screen ]]
then
    MPLAYER_PROFILE="audio"
else
    MPLAYER_PROFILE="console"
fi

if [[ -f ~/.zshaliases ]]
then
    source ~/.zshaliases
fi

alias mplayer="mplayer -profile $MPLAYER_PROFILE"

if [[ -f /usr/bin/vim && -x /usr/bin/vim ]]
then
    alias vi=vim
else
    unalias vi
fi

if [[ -f ~/.zshfunctions ]]
then
    source ~/.zshfunctions
fi
