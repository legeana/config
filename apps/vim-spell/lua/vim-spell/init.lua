local M = {}

M.config = {
	main = nil,
	extras = {},
}

function M.setup(opts)
	M.config = vim.tbl_deep_extend("force", M.config, opts or {})

	local opt = vim.opt
	if M.config.main == nil then
		return
	end
	opt.spell = true
	opt.spelllang = "en"
	opt.spelloptions = "noplainbuffer,camel"
	opt.spellfile:prepend(M.config.main)
	for _, extra in ipairs(M.config.extras) do
		opt.spellfile:append(extra)
	end
	-- TestSomething
	vim.api.nvim_set_hl(0, "SpellBad", {
		cterm = { underline = true },
		ctermfg = "red",
		undercurl = false,
		underline = true,
		fg = "red",
	})
end

vim.api.nvim_create_user_command("RegenSpellFiles", function()
	local spellfiles = vim.opt.spellfile:get()
	for _, spellfile in ipairs(spellfiles) do
		vim.cmd("mkspell! " .. spellfile)
	end
end, { desc = "Regenerate all spell files" })

return M
