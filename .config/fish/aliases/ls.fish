if command ls --version >/dev/null 2>&1
    alias ls='ls --color=auto --human-readable'
    alias lls='ls --color=always --human-readable'
else
    # Assuming BSD/OSX version
    alias ls='ls -G'
    alias lls='ls -G -h'
end

alias la='ls -A'
alias ll='ls -l'
alias lsa='ls -Al'
alias lsd='lsa -d'
