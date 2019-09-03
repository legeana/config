function git-push-to-all
    git remote | parallel git push "{}"
end
