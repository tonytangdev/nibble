# History is a bounded cache, not a source of truth

Claude Code prunes its logs after about 30 days, and Still hungry and Sleepy need past Limit readings, so nibble keeps its own store: per-hour aggregates (hourly since ADR-0007, so the weekly reset lands exactly) and read bookmarks for 13 weeks, plus 31 days of Limit readings (enough for Sleepy's peak over the Rolling 30 days), pruned on every write. It holds no prompt or reply text. Deleting it loses only what Claude Code has already pruned; everything else is rebuilt from the logs on next launch.
