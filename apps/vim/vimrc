let s:xdg_config_home = empty($XDG_CONFIG_HOME) ? $HOME . '/.config' : $XDG_CONFIG_HOME
let s:nvim_dir = s:xdg_config_home . '/nvim'

execute 'set runtimepath^=' . fnameescape(s:nvim_dir)
execute 'set runtimepath^=' . fnameescape(s:nvim_dir . '/after')
execute 'source ' . fnameescape(s:nvim_dir . '/init.vim')
