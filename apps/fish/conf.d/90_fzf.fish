set -gx FZF_DEFAULT_OPTS (string join ' ' -- \
    --cycle \
    --layout=reverse \
    --border \
    --height=90% \
    --preview-window=wrap \
    --marker='"*"' \
    --bind=ctrl-j:down,ctrl-k:up
)
