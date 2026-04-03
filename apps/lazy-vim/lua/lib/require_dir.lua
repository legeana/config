local Path = require("plenary.path")
local scandir = require("plenary.scandir")
local base_path = Path:new(vim.fn.stdpath("config"), "lua")

-- Loads a directory, non-recursively.
return function(package)
  local pkg_path = base_path
  for part in vim.gsplit(package, ".", { plain = true }) do
    pkg_path = pkg_path:joinpath(part)
  end
  local files = scandir.scan_dir(pkg_path.filename, {
    hidden = false,
    add_dirs = false,
    search_pattern = "%.lua$",
  })
  for _, file in ipairs(files) do
    local rel = Path:new(file):make_relative(base_path.filename)
    local pkg = rel:gsub("%.lua$", ""):gsub(Path.path.sep, ".")
    require(pkg)
  end
end
