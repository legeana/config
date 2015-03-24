export ZSH_LOADED="$ZSH_LOADED:USER_PROFILE"

umask 022

# Use hard limits, except for a smaller stack and no core dumps
unlimit
limit stack 8192
limit core 0
limit maxproc 4096
limit -s

export PATH="$HOME/bin:$PATH"
export PATH="$GOPATH/bin:$PATH"
export PATH="$PATH:$HOME/.gem/ruby/2.2.0/bin"
export GOPATH="$HOME/.go"
export MPD_HOST="lex-pc" MPD_PORT="6600"
export EDITOR="vim" PAGER="most"
export LC_NUMERIC="C"

export ZPROFILE_LOADED=1
