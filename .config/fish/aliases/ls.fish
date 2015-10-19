function ls
    command ls --color=auto --human-readable $argv
end

function la
    ls -A $argv
end

function ll
    ls -l $argv
end

function lls
    ls --color=always --human-readable $argv
end

function lsa
    ls -Al $argv
end

function lsd
    lsa -d $argv
end
