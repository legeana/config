function trace-fish
    if count $argv >/dev/null 2>&1
        fish_trace=1 __trace_fish_wrapper $argv
    else
        echo 'Use trace-fish function arg1 arg2...' >&2
        echo 'or fish_trace=1 function arg1 arg2...' >&2
    end
end

function __trace_fish_wrapper \
    --description 'Wraps $argv one level deeper to trace the $argv invocation itself'
    $argv
end
