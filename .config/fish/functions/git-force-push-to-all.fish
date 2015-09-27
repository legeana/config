function git-force-push-to-all
    set -l h (git log HEAD^..HEAD --pretty='%H')
    set -l hs (git log HEAD^..HEAD --pretty='%h')
    set -l h_
    echo -n "Enter hash to force push: "
    read h_
    if [ $h_ != $h && $h_ != $hs ]
        echo "Hash mismatch" >&2
        return 1
    end
    git remote | parallel git push -f "{}"
end
