# Limit readings come from Claude Code's cached utilization

Claude Code caches account-wide Utilization for the 5-hour and 7-day Limit windows in `~/.claude.json` (`cachedUsageUtilization`). It is undocumented and only refreshes while Claude Code runs, which is when limits matter. We read it instead of starting Claude Code's control protocol on a timer, as Vorssaint does, because that is far more machinery for little gain. If the cache is missing or its shape changes, Still hungry turns off and nothing else breaks. Revisit if readings prove stale.

Note: Utilization is account-wide (claude.ai included) while API value is Claude Code only.
