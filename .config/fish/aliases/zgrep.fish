function zgrep
    command zgrep --color=auto $argv
end

function zegrep
    zgrep -E $argv
end

function zfgrep
    zgrep -F $argv
end

function zrgrep
    zgrep -R $argv
end

function pcregrep
    pcregrep --color=auto $argv
end
