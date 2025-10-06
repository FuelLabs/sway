#!/bin/bash
echo '<table><thead>'
head -n 1 "$1" | sed -e 's/^/<tr><th>/' -e 's/,/<\/th><th>/g' -e 's/$/<\/th><\/tr>/'
echo '</thead><tbody>'
tail -n +2 "$1" | sed -e 's/^/<tr><td>/' -e 's/,/<\/td><td>/g' -e 's/$/<\/td><\/tr>/'
echo "</tobyd></table>"