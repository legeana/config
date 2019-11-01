function is_local_filesystem -a path
    if test -z $path
        set path $PWD
    end
    set -l fstype (stat --file-system  --format='%T' $path)
    switch $fstype
        case 'fuseblk'
            return 1
        case '*'
            return 0
    end
end
