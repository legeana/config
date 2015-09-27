function git-version
    echo (git rev-list --count HEAD).(git rev-parse --short HEAD)
end
