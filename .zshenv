# Global shell settings

export ZSH_LOADED="$ZSH_LOADED:USER_ENV"

emulate sh -c 'source /etc/profile'

# Do not load system-wide configuration.
# They may override local settings.
setopt noglobal_rcs

export GOPATH="$HOME/.go"

typeset -U path cdpath fpath manpath
path=("$HOME/bin" $path)
path=("$GOPATH/bin" $path)

umask 022

# Use hard limits, except for a smaller stack and no core dumps
unlimit
limit stack 8192
limit core 0
limit maxproc 4096
limit -s

export EDITOR="vim" PAGER="most"
export LC_NUMERIC="C"

if [[ -f ~/.zshlocalenv ]]
then
    source ~/.zshlocalenv
fi

export ZSHENV_LOADED=1
