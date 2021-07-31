function update-my-configs
    set -l config (dirname (realpath ~/.config))
    set -l gitcmd git --work-tree=$config --git-dir=$config/.git
    eval $gitcmd pull --ff-only
    eval $config/setup
end
