call autoplug#begin('~/.config/nvim-plugins')
Plug 'tpope/vim-sensible'
Plug 'editorconfig/editorconfig-vim'
Plug 'kamykn/spelunker.vim'
call autoplug#end()

" Force tpope/vim-sensible to load now.
" Otherwise it overrides vimrc and init.vim.
runtime! plugin/sensible.vim
