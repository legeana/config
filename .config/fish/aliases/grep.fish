function grep --wraps=grep
    command grep --color=auto $argv
end

function egrep --wraps=grep
    grep -E $argv
end

function fgrep --wraps=grep
    grep -F $argv
end

function pcregrep --wraps=grep
    command pcregrep --color=auto $argv
end
