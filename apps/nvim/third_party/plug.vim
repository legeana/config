let s:xdg_cache_home = empty($XDG_CACHE_HOME) ? $HOME . '/.cache' : $XDG_CACHE_HOME
let s:plug_dir = s:xdg_cache_home . '/vim-plug'
execute 'set runtimepath+=' . fnameescape(s:plug_dir)

call plug#begin(s:plug_dir . '/plugins')
Plug 'tpope/vim-sensible'
Plug 'editorconfig/editorconfig-vim'
Plug 'machakann/vim-highlightedyank'
Plug 'dag/vim-fish'
Plug 'itchyny/lightline.vim'
Plug 'dracula/vim', { 'as': 'dracula' }
Plug 'rust-lang/rust.vim'

if executable("node")
    Plug 'neoclide/coc.nvim', {'branch': 'release'}
    runtime! opt_plugin/coc.vim
endif

call plug#end()

" Force tpope/vim-sensible to load now.
" Otherwise it overrides vimrc and init.vim.
runtime! plugin/sensible.vim
