function verbose-eval
    echo '$' $argv
    sh -c '$@' '' $argv
end
