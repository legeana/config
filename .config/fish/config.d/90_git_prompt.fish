set -U __fish_git_prompt_showdirtystate yes
set -U __fish_git_prompt_showstashstate yes
set -U __fish_git_prompt_showuntrackedfiles yes
set -U __fish_git_prompt_showupstream yes

set -U __fish_git_prompt_color_branch yellow
set -U __fish_git_prompt_color_upstream_ahead green
set -U __fish_git_prompt_color_upstream_behind red

# Clean previously incorrectly used universal variables.
function __clean_fish_git_prompt
    set -e $argv[1]
    set -g $argv[1] $argv[2]
end

__clean_fish_git_prompt __fish_git_prompt_char_cleanstate '✔'
__clean_fish_git_prompt __fish_git_prompt_char_dirtystate '*' '✚'
__clean_fish_git_prompt __fish_git_prompt_char_invalidstate '#' '✖'
__clean_fish_git_prompt __fish_git_prompt_char_stagedstate '+' '●'
__clean_fish_git_prompt __fish_git_prompt_char_stashstate '$'
__clean_fish_git_prompt __fish_git_prompt_char_stateseparator ' ' '|'
__clean_fish_git_prompt __fish_git_prompt_char_untrackedfiles '%' '…'
__clean_fish_git_prompt __fish_git_prompt_char_upstream_ahead '>' '↑'
__clean_fish_git_prompt __fish_git_prompt_char_upstream_behind '<' '↓'
__clean_fish_git_prompt __fish_git_prompt_char_upstream_diverged '<>'
__clean_fish_git_prompt __fish_git_prompt_char_upstream_equal '='
__clean_fish_git_prompt __fish_git_prompt_char_upstream_prefix ''

functions --erase __clean_fish_git_prompt
