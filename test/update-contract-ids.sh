#! /bin/bash

print_help() {
    echo "
 -i          run interactively
";
}

INTERACTIVELY="0"

while [[ "$#" -gt 0 ]]; do
    case $1 in
        -i) INTERACTIVELY="1"; shift ;;
        *) print_help; exit 1 ;;
    esac
    shift
done

if [[ "$INTERACTIVELY" == "0" ]]; then
  CHANGES=$(git status --porcelain | wc -l)
  if [ "$CHANGES" != "0" ]; then
    echo "git state is not clean. commit or restore first."
    exit
  fi
fi

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

replace () {
    if [ $# -lt 2 ]
    then
        echo "Recursive, interactive text replacement"
        echo "Usage: replace text replacement"
        return
    fi

    vim -u NONE -c ":execute ':argdo %s/$1/$2/gc | update' | :q" $(ag $1 -l)
}

grep --include \*.sw -Hno "// AUTO-CONTRACT-ID" . -R | while read line ; do
    PARTS=($(echo $line | sed 's/:/ /g'))
    FOLDER=$(dirname ${PARTS[0]})
    FILE=${PARTS[0]}
    LINE=${PARTS[1]}

    SED_COMMAND="${LINE}"'!d'
    CONTRACT_ARGS=($(sed "$SED_COMMAND" $FILE))
    CONTRACT_ARGS=$(join_by " " ${CONTRACT_ARGS[@]:6})

    if [[ $CONTRACT_ARGS ]]; then 
        PROJ=$(realpath "$FOLDER/..")
        echo -e "${BOLD_WHITE}$PROJ${NC}"

        pushd "$FOLDER/.." >> /dev/null
        CONTRACT_ID=$(cargo r -p forc --release -- contract-id --path $CONTRACT_ARGS 2> /dev/null | grep -oP '0x[a-zA-Z0-9]{64}')

        if [[ $CONTRACT_ID ]]; then 
            popd >> /dev/null

            if [[ "$INTERACTIVELY" == "0" ]]; then
              sed -i "${LINE}s/0x[a-zA-Z0-9]*/$CONTRACT_ID/g" $FILE
              echo -e "    ${BOLD_GREEN}ok${NC} ($CONTRACT_ID)"
            else
              echo "$FILE" | xargs -o vim -n -c ":${LINE}s/0x[a-zA-Z0-9]*/$CONTRACT_ID/gc" -c ":x"
            fi
        else
            echo -e "    ${BOLD_RED}error${NC}"
            cargo r -p forc --release -- contract-id --release
            popd >> /dev/null
        fi
    fi
done
