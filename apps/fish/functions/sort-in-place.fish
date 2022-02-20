function sort-in-place -d 'Sort file in-place'
    sort $argv | sponge (get-arguments $argv)
end
