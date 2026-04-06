local Path = require("plenary.path")
local scandir = require("plenary.scandir")
local base_path = Path:new(vim.fn.stdpath("config"), "lua")

--- Loads a directory, non-recursively.
-- @param package: the name of the directory package, e.g. "foo.bar"
-- @param opts: optional configuration
--   opts.depth (int): how deep to search files for, defaults to 1
--   opts.allow_missing (bool): allow the directory itself to be missing, defaults to false
-- @return nothing
return function(package, opts)
  opts = opts or {}
  opts.depth = opts.depth or 1
  opts.allow_missing = opts.allow_missing or false

  local pkg_path = base_path
  for part in vim.gsplit(package, ".", { plain = true }) do
    pkg_path = pkg_path:joinpath(part)
  end

  if opts.allow_missing and not pkg_path:exists() then
    return
  end

  local files = scandir.scan_dir(pkg_path.filename, {
    hidden = false,
    add_dirs = false,
    search_pattern = "%.lua$",
    depth = opts.depth,
  })
  for _, file in ipairs(files) do
    local rel = Path:new(file):make_relative(base_path.filename)
    local pkg = rel:gsub("%.lua$", ""):gsub(Path.path.sep, ".")
    require(pkg)
  end
end
