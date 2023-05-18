set -g fisher_path ~/.local/share/fisher
load_plugin $fisher_path

function __get_fisher
    curl -sL https://raw.githubusercontent.com/jorgebucaran/fisher/main/functions/fisher.fish |
        source &&
        fisher install jorgebucaran/fisher
end

if ! functions -q fisher
    function fisher
        __get_fisher
        load_plugin $fisher_path
        fisher $argv
    end
end
