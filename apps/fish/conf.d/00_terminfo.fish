must-have-command infocmp

function is_terminfo_available
    infocmp $argv >/dev/null 2>&1
end

if is_terminfo_available
    exit
end

# Workarounds.
set -l term
for term in xterm-256color xterm-color xterm
    if is_terminfo_available $term
        echo $TERM is not available, using $term instead! >&2
        set -gx TERM $term
        exit
    end
end

echo $TERM may not be available
