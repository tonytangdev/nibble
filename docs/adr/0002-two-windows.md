# Break-even follows the Billing cycle; Sleepy uses Rolling 30 days

Status: break-even now follows the Week instead (ADR-0007). The Rolling 30 days and full-days rules below still hold.

A Billing cycle matches the real bill and gives one clean Paid off moment per cycle, as the Wedge refills on renewal. Early in every cycle it reads near zero, though, which would make Sleepy fire falsely. So the downgrade judgement uses Rolling 30 days (needing at least 14 days of history, and scaling shorter history up to 30 days) while the Wedge, Multiplier and Paid off use the Billing cycle. Vorssaint uses rolling 30 days for everything; we rejected that because break-even would flap and there would be no refill moment.

Both Sleepy and Still hungry look only at full days before today, so a Mood changes at most once a day instead of flapping as readings arrive.
