#!/bin/bash

FILES="./test_*/*"
MANIFEST="Forc.toml"

pwd
for f in $FILES
do
if [ -e ${f}/$MANIFEST ]
  then
  if [ -d "${f}" ];
    then
    echo "building test $f..."
      forc build -o temp -p $f
      echo ✔
      if ! [ -f temp ];
        then
        echo  "❌  Failed to build $f"
        exit 1
      fi
      rm temp
  fi
fi
done
echo "Successfully built all projects."
