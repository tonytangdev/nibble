# History is a bounded cache, not a source of truth

Claude Code prunes its logs after about 30 days, and Still hungry needs past Limit readings, so nibble keeps its own store: per-day aggregates and read bookmarks for 13 weeks, plus 14 days of Limit readings, pruned on every write. It holds no prompt or reply text. Deleting it loses only what Claude Code has already pruned; everything else is rebuilt from the logs on next launch.
