function ls --wraps=ls
    command ls --color=auto --human-readable $argv
end

function la --wraps=ls
    ls -A $argv
end

function ll --wraps=ls
    ls -l $argv
end

function lls --wraps=ls
    ls --color=always --human-readable $argv
end

function lsa --wraps=ls
    ls -Al $argv
end

function lsd --wraps=ls
    lsa -d $argv
end
