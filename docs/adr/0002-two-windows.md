# Break-even follows the Billing cycle; Sleepy uses Rolling 30 days

A Billing cycle matches the real bill and gives one clean Paid off moment per cycle, as the Wedge refills on renewal. Early in every cycle it reads near zero, though, which would make Sleepy fire falsely. So the downgrade judgement uses Rolling 30 days (needing at least 14 days of history) while the Wedge, Multiplier and Paid off use the Billing cycle. Vorssaint uses rolling 30 days for everything; we rejected that because break-even would flap and there would be no refill moment.
