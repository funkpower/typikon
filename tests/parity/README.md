# Parity

Every binding answers the same question, so every binding has to give the same
answer. These probes ask each one for the same twelve days and compare all
twenty-two fields against what the command line says.

The probes are chosen for the places the bindings could plausibly disagree:
Pascha itself, the Nativity on both calendars, a Wednesday of the Great Fast
under Byzantine Catholic law and again at the Typikon, a weekday of the
Nativity Fast where Greek and Slavic usage part, the one Orthodox church on the
Gregorian Paschalion, and a day with no feast at all.

Run them with `tests/parity.sh`. Surfaces whose toolchain is absent are skipped
by name rather than silently passed.
