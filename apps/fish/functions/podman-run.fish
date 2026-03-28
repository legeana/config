function podman-run
    argparse 'h/help' 'm/mount=*' -- $argv
    or return

    set -l mounts
    for mount in $_flag_mount
        set -a mounts "--volume=$mount:$mount:Z"
    end

    podman run \
        --userns=keep-id:uid=(id -u),gid=(id -g) \
        --env-host \
        --workdir="$PWD" \
        --interactive \
        --tty \
        --rm \
        --network=host \
        $mounts \
        $argv
end
