call autoplug#begin('~/.local/share/nvim-plug/plugins')
Plug 'tpope/vim-sensible'
Plug 'editorconfig/editorconfig-vim'
Plug 'machakann/vim-highlightedyank'
Plug 'kamykn/spelunker.vim'
Plug 'dag/vim-fish'
Plug 'neoclide/coc.nvim', {'branch': 'release'}
call autoplug#end()

" Force tpope/vim-sensible to load now.
" Otherwise it overrides vimrc and init.vim.
runtime! plugin/sensible.vim
