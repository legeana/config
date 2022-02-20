function get-arguments -d 'Return arguments from a command line without --'
    set allargs ''
    for arg in $argv
        if test -n "$allargs"
            echo -- "$arg"
        else if test "$arg" = '--'
            set allargs 1
        else if ! string match --quiet --regex '^-' -- "$arg"
            echo -- "$arg"
        end
    end
end
