function forget-last -d 'Forget last command'
    history delete --case-sensitive --exact (history search --max=1)
end
