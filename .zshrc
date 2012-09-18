# загружаем дефолтный профиль оболочки
source /etc/profile

# Установка атрибутов доступа для вновь создаваемых файлов
umask 022

# Use hard limits, except for a smaller stack and no core dumps
unlimit
limit stack 8192
limit core 0
limit -s

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

# заголовки и прочее.

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
    cmd="$( echo "$1" | head -n1 | sed -r 's/^(sudo [^[:space:]]+|[^[:space:]]+).*/\1/' )"
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

# экранируем спецсимволы в url, например &, ?, ~ и так далее
autoload -U url-quote-magic
zle -N self-insert url-quote-magic

# модуль для переименования файлов
autoload zmv

# менюшку нам для автокомплита
zstyle ':completion:*' menu yes select

# Позволяем разворачивать сокращенный ввод, к примеру cd /u/sh в /usr/share
autoload -U compinit && compinit

# The following lines were added by compinstall

#zstyle ':completion:*' completer _expand _complete _ignored _correct _approximate
#zstyle ':completion:*' completer _expand _complete _ignored _approximate
#zstyle ':completion:*' completer _expand
#zstyle ':completion:*' completer _complete _ignored
#zstyle :compinstall filename '/home/lex/.zshrc'

autoload -Uz compinit
compinit
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
# End of lines configured by zsh-newuser-install

if [[ -x /usr/bin/dircolors ]]
then
    eval "`dircolors -b`"
fi

#autoload -U promptinit
#promptinit
#prompt suse

#non-colorful
#export PROMPT='%(!..%n@)%m:%~%(!.#.$) '
#export RPROMPT='%? %U%T%u %U%w%u'
#export SPROMPT='zsh: Заменить '\''%R'\'' на '\''%r'\'' ? [Yes/No/Abort/Edit] '
#colorful
export PROMPT='%(!..%F{magenta}%n%f@)%F{cyan}%m%f:%~%(?.%F{green}.%F{red})%(!.#.$)%f '
export RPROMPT='%(?.%F{green}.%F{red})%?%f %F{cyan}%U%T%u %U%w%u%f'
export SPROMPT='zsh: Заменить '\''%F{red}%R%f'\'' на '\''%F{green}%r%f'\'' ? [%F{green}Yes%f/%U%F{red}No%f%u/Abort/%F{blue}Edit%f] '

bindkey '^[[1;5D' backward-word
bindkey '^[[1;5C' forward-word
bindkey '^[[5~' history-search-backward
bindkey '^[[6~' history-search-forward

if [[ -f ~/.zshinputrc ]]
then
    source ~/.zshinputrc
fi

typeset -U path cdpath fpath manpath

#dot()
#{
#    if [[ $LBUFFER = *.. ]]
#    then
#        LBUFFER+=/..
#    else
#       LBUFFER+=.
#    fi
#}
#autoload -U dot
#zle -N dot
#bindkey . dot

if [[ -d $HOME/bin ]]
then
    export PATH="$HOME/bin:$PATH"
fi

ulimit -u 4096
