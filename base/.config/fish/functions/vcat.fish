function vcat
    for i in $argv
        set -l lines (wc -l < $i)
        if math "$lines <= 1" >/dev/null
            echo "$i: "(cat "$i")
        else
            echo "{{{[$i]"
            cat $i
            echo "}}}[$i]"
        end
    end
end
