return {
	"",
	name = "vim-spell",
	event = "VeryLazy",
	config = function(plugin, opts)
		local root = plugin.dir
		opts.main = root .. "/draft/en.utf-8.add"
		opts.extras = {
			root .. "/committed/en.utf-8.add",
		}
		require("vim-spell").setup(opts)
	end,
}
