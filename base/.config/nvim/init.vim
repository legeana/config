if filereadable(expand('~/.config/nvim/before.vim'))
    source ~/.config/nvim/before.vim
endif

call autoplug#begin('~/.config/nvim-plugins')
Plug 'sheerun/vim-polyglot'
Plug 'tpope/vim-sensible'
call autoplug#end()
