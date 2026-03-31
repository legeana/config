if ! string match --quiet --regex ghostty $TERM
    exit
end
if ! set -q GHOSTTY_RESOURCES_DIR
    exit
end

load_d $GHOSTTY_RESOURCES_DIR/shell-integration/fish/vendor_conf.d
