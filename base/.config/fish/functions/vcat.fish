function vcat
    for i in $argv
        set -l lines (wc -l < $i)
        if test "$lines" -le 1
            echo "$i:" (cat "$i")
        else
            echo "{{{[$i]"
            cat $i
            echo "}}}[$i]"
        end
    end
end
