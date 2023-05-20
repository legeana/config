call autoplug#begin()
Plug 'tpope/vim-sensible'
Plug 'editorconfig/editorconfig-vim'
Plug 'machakann/vim-highlightedyank'
Plug 'kamykn/spelunker.vim'
Plug 'dag/vim-fish'
Plug 'itchyny/lightline.vim'
Plug 'dracula/vim', { 'as': 'dracula' }

if executable("node")
    Plug 'neoclide/coc.nvim', {'branch': 'release'}
    runtime! opt_plugin/coc.vim
endif

call autoplug#end()

" Force tpope/vim-sensible to load now.
" Otherwise it overrides vimrc and init.vim.
runtime! plugin/sensible.vim
