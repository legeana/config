function kanidm-tools
    podman-run \
        --mount=$HOME/.config/kanidm \
        --mount=$HOME/.cache/kanidm_tokens \
        kanidm/tools:latest kanidm \
        $argv
end
