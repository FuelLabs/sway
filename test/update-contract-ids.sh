#!/usr/bin/env bash

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


CHANGES=$(git status --porcelain | wc -l)

BOLD_RED='\033[1;31m'
BOLD_GREEN="\033[1;32m"
BOLD_YELLOW='\033[1;33m'
BOLD_WHITE='\033[1;97m'
NC='\033[0m'

# macOS compatibility: ggrep and gsed from `brew install gnu-sed grep`
if [ -x "$(command -v ggrep)" ]; then
  grep="ggrep"
else
  grep="grep"
fi
if [ -x "$(command -v gsed)" ]; then
  sed="gsed"
else
  sed="sed"
fi

function join_by {
  local d=${1-} f=${2-}
  if shift 2; then
    printf %s "$f" "${@/#/$d}"
  fi
}

ask_confirmation() {
    echo -e "${BOLD_WHITE}Command below needs confirmation before running.${NC} $2"
    echo "> $1"
    read -p "Run the command above? (y/n) " -n 1 -r </dev/tty
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]
    then
        bash -c "$1"
    else
        exit 1
    fi
}

$grep --include \*.sw -Hno "// AUTO-CONTRACT-ID" . -R | while read line ; do
    PARTS=($(echo $line | $sed 's/:/ /g'))
    FOLDER=$(dirname ${PARTS[0]})
    FILE=${PARTS[0]}
    LINE=${PARTS[1]}

    SED_COMMAND="${LINE}"'!d'
    CONTRACT_ARGS=($($sed "$SED_COMMAND" $FILE))
    CONTRACT_ARGS=$(join_by " " ${CONTRACT_ARGS[@]:6})

    if [[ $CONTRACT_ARGS ]]; then 
        PROJ=$(realpath "$FOLDER/..")
        echo -e "${BOLD_WHITE}$PROJ${NC}"

        pushd "$FOLDER/.." >> /dev/null
        CONTRACT_ID=$(/home/xunilrj/sway/target/release/forc contract-id --path $CONTRACT_ARGS 2> /dev/null | $grep -oP '0x[a-zA-Z0-9]{64}')

        if [[ $CONTRACT_ID ]]; then 
            popd >> /dev/null

            SED_EXPR="${LINE}s/0x[a-zA-Z0-9]*/$CONTRACT_ID/g"

            # check if there is a diff
            diff -s --color <(cat $FILE) <(cat $FILE | $sed --expression="$SED_EXPR") > /dev/null
            if [ $? -eq 0 ]; then
              # no diff, continue
              echo -e "    ${BOLD_GREEN}no changes needed${NC} ($CONTRACT_ID)"
            else 
              # diff detected, check we are clean to update files, if not abort
              if [[ "$INTERACTIVELY" == "0" ]]; then
                # Don´t change anything if git is dirty
                if [ "$CHANGES" != "0" ]; then
                  echo -e "    ${BOLD_RED}Aborting${NC} This contract id needs update, but git state is not clean. commit, restore first or run with \"-i\"."
                  echo $FILE
                  diff -s --color <(cat $FILE) <(cat $FILE | $sed --expression="$SED_EXPR")
                  exit
                fi
                # we are clean and can update files
                $sed -i "$SED_EXPR" $FILE
              else
                # ask confirmation before applying the change
                diff -s --color <(cat $FILE) <(cat $FILE | $sed --expression="$SED_EXPR")
                ask_confirmation "$sed -i \"$SED_EXPR\" $FILE" "Update contract id"
              fi
              echo -e "    ${BOLD_GREEN}updated${NC} ($CONTRACT_ID)"
            fi
        else
            echo -e "    ${BOLD_RED}error${NC}"
            /home/xunilrj/sway/target/release/forc contract-id --release
            popd >> /dev/null
        fi
    fi
done
