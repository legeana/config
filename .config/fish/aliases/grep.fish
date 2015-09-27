function grep
    command grep --color=auto $argv
end

function egrep
    grep -E $argv
end

function fgrep
    grep -F $argv
end

function pcregrep
    pcregrep --color=auto $argv
end
