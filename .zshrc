export ZSH_LOADED="$ZSH_LOADED:USER_RC"

# Autoload zsh modules when they are referenced
zmodload -a zsh/stat stat
zmodload -a zsh/zpty zpty
zmodload -a zsh/zprof zprof
zmodload -ap zsh/mapfile mapfile

# Completions
zstyle ':completion:*::::' completer _expand _complete _ignored _approximate
zstyle -e ':completion:*:approximate:*' max-errors 'reply=( $(( ($#PREFIX+$#SUFFIX)/3 )) numeric )'
zstyle ':completion:*:expand:*' tag-order all-expansions
zstyle ':completion:*' verbose yes
zstyle ':completion:*:descriptions' format '%B%d%b'
zstyle ':completion:*:messages' format '%d'
zstyle ':completion:*:warnings' format 'No matches for: %d'
zstyle ':completion:*:corrections' format '%B%d (errors: %e)%b'
zstyle ':completion:*' group-name ''
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'
zstyle ':completion:*:*:-subscript-:*' tag-order indexes parameters
zstyle ':completion:*:*:(^rm):*:*files' ignored-patterns '*?.o' '*?.c~''*?.old' '*?.pro'
zstyle ':completion:*:functions' ignored-patterns '_*'

zstyle ':completion:*:*:kill:*:processes' list-colors "=(#b) #([0-9]#)*=$color[cyan]=$color[red]"

# Headers etc...

precmd()
{
    [[ -t 1 ]] || return
    case "$TERM" in
        *xterm*|rxvt|(dt|k|E|a)term*) print -Pn "\e]0;[%~] %m\a"    ;;
        screen(-bce|.linux)) print -Pn "\ek[%~]\e\\" && print -Pn "\e]0;[%~] %m (screen)\a" ;;
    esac
    # end of command
    echo -ne '\a'
}

preexec()
{
    [[ -t 1 ]] || return
    local cmd="$( echo "$1" | head -n1 | sed -r 's/^(sudo [^[:space:]]+|[^[:space:]]+).*/\1/' )"
    case "$TERM" in
        *xterm*|rxvt|(dt|k|E|a)term*) print -Pn "\e]0;<$cmd> [%~] %m\a" ;;
        screen(-bce|.linux)) print -Pn "\ek<$cmd> [%~]\e\\" && print -Pn "\e]0;<$cmd> [%~] %m (screen)\a" ;;
    esac
}

chpwd()
{
    if [[ -d .git ]]
    then
        git status
    elif [[ -d .svn ]]
    then
        svn status
    fi
}

typeset -g -A key

# Escape URLs
autoload -U url-quote-magic
zle -N self-insert url-quote-magic

# Smart renaming
autoload zmv

# Autocomplete menu
zstyle ':completion:*' menu yes select

# Expand /u/sh into /usr/share
autoload -Uz compinit && compinit

# End of lines added by compinstall
# Lines configured by zsh-newuser-install
HISTFILE=~/.histfile
HISTSIZE=1000
SAVEHIST=1000
setopt appendhistory autocd beep notify hist_verify histignoredups correctall ignoreeof
#no_clobber
#setopt histignorealldups histignorespace
unsetopt extendedglob nomatch
bindkey -e

if [[ -x /usr/bin/dircolors ]]
then
    eval "`dircolors -b`"
fi

#autoload -U promptinit
#promptinit
#prompt suse

export PROMPT='%(!..%F{magenta}%n%f@)%F{cyan}%m%f:%~%(?.%F{green}.%F{red})%(!.#.$)%f '
export RPROMPT='%(?.%F{green}.%F{red})%?%f %F{cyan}%U%T%u %U%w%u%f'
export SPROMPT='zsh: Replace '\''%F{red}%R%f'\'' by '\''%F{green}%r%f'\'' ? [%F{green}Yes%f/%U%F{red}No%f%u/Abort/%F{blue}Edit%f] '

bindkey '^[[1;5D' backward-word
bindkey '^[[1;5C' forward-word
bindkey '^[[5~' history-search-backward
bindkey '^[[6~' history-search-forward

if [[ -f ~/.zshinputrc ]]
then
    source ~/.zshinputrc
fi

typeset -U path cdpath fpath manpath

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

if [[ -x /usr/bin/grc ]]
then
    alias grc='grc --colour=auto'
    alias ping='grc ping'
    alias last='grc last'
    alias netstat='grc netstat'
    alias traceroute='grc traceroute'
fi

if [[ -f ~/.zshfunctions ]]
then
    source ~/.zshfunctions
fi

export ZSHRC_LOADED=1
