#! /bin/bash

# CHANGES=$(git status --porcelain | wc -l)
# if [ "$CHANGES" != "0" ]; then
#   echo "git state is not clean. commit or restore first."
#   exit
# fi

BOLD_RED='\033[1;31m'
BOLD_GREEN="\033[1;32m"
BOLD_YELLOW='\033[1;33m'
BOLD_WHITE='\033[1;97m'
NC='\033[0m'

function join_by {
  local d=${1-} f=${2-}
  if shift 2; then
    printf %s "$f" "${@/#/$d}"
  fi
}

grep --include \*.sw -Hno "// AUTO-CONTRACT-ID" . -R | while read line ; do
    PARTS=($(echo $line | sed 's/:/ /g'))
    FOLDER=$(dirname ${PARTS[0]})
    FILE=${PARTS[0]}
    LINE=${PARTS[1]}

    CONTRACT_ARGS=($(sed "$LINE!d" $FILE))
    CONTRACT_ARGS=$(join_by " " ${CONTRACT_ARGS[@]:6})

    if [[ $CONTRACT_ARGS ]]; then 
        PROJ=$(realpath "$FOLDER/..")
        echo -e "${BOLD_WHITE}$PROJ${NC}"

        pushd "$FOLDER/.." >> /dev/null
        CONTRACT_ID=$(cargo r -p forc --release -- contract-id --path $CONTRACT_ARGS 2> /dev/null | grep -oP '0x[a-zA-Z0-9]{64}')

        if [[ $CONTRACT_ID ]]; then 
            popd >> /dev/null
            sed -i "${LINE}s/0x[a-zA-Z0-9]*/$CONTRACT_ID/g" $FILE
            echo -e "    ${BOLD_GREEN}ok${NC} ($CONTRACT_ID)"
        else
            echo -e "    ${BOLD_RED}error${NC}"
            cargo r -p forc --release -- contract-id --release
            popd >> /dev/null
        fi
    fi
done
