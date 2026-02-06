function vecho -d 'Print each argument on a separate line'
    for arg in $argv
        echo -- $arg
    end
end
