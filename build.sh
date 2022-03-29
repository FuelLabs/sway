#!/bin/bash

# Place in root of project and run to build the project and all its tests and artifacts
FILES="./sway-lib-std/tests/test_*/*"
for f in $FILES
do
if [ -d "${f}" ];
then
echo "building test $f..."
  /Users/furnic/dev/fuel/sway/target/debug/forc build -o temp -p $f
  if ! [ -f temp ];
  then
  echo "Failed to build $f"
  exit 1
  fi
  rm temp
fi
done

echo "Successfully built all projects."
