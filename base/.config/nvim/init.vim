for s:third_party in expand('~/.config/nvim/third_party/**/*.vim', 0, 1)
    execute 'source ' . fnameescape(s:third_party)
endfor
