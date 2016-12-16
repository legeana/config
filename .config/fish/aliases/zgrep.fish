function zgrep --wraps=zgrep
    command zgrep --color=auto $argv
end

function zegrep --wraps=zgrep
    zgrep -E $argv
end

function zfgrep --wraps=zgrep
    zgrep -F $argv
end

function zrgrep --wraps=zgrep
    zgrep -R $argv
end
