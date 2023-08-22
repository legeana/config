function __fish_prompt_cmd_saver --on-event fish_postexec
    if string length --quiet $argv[1]
        set -l arg $argv[1]
        # Use quoted substitution since commands may return multiple tokens.
        set -l arg (string replace \n ' ' "$arg")
        set -l arg (string replace -r '^\s*(\S+)\s.*$' '$1' "$arg")
        set -gx PROMPT_LAST_COMMAND (string replace -r '^([^ ]*/)?([^/ ]+)(\s.*)?$' '$2' $arg)
    end
end
