#!/bin/bash

FILES="./test_*/*"
MANIFEST="forc.toml"

pwd
for f in $FILES
do
if [ ! -e ${f}/$MANIFEST ]
  then
  echo "Not building $f, no manifest found."
  else
  if [ -d "${f}" ];
    then
    echo "building test $f..."
      forc build -o temp -p $f
      if ! [ -f temp ];
        then
        echo "Failed to build $f"
        exit 1
      fi
      rm temp
  fi
fi
done

echo "Successfully built all projects."
