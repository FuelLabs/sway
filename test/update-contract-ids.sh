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

return_only_after_item() {
    local target="$1"
    shift
    local arr=("$@")
    local new_arr=()
    local found=false

    for item in "${arr[@]}"; do
        if [[ "$item" == "$target" ]]; then
            found=true
            continue
        fi
        if $found; then
            new_arr+=("$item")
        fi
    done

    echo "${new_arr[@]}"
}

get_new_contract_id() {
    line="$1"
    PARTS=($(echo $line | $sed 's/:/ /g'))
    FILE=${PARTS[0]}
    LINE=${PARTS[1]}
    DIR=$(dirname $FILE)

    >&2 echo -e "${BOLD_WHITE}$DIR${NC}"

    SED_COMMAND="${LINE}"'!d'
    CONTRACT_ARGS=($($sed "$SED_COMMAND" $FILE))
    CONTRACT_ARGS=($(return_only_after_item "AUTO-CONTRACT-ID" "${CONTRACT_ARGS[@]}"))
    CONTRACT_ARGS=$(join_by " " ${CONTRACT_ARGS[@]})
    >&2 echo -e "    $CONTRACT_ARGS"

    if [[ $CONTRACT_ARGS ]]; then
        PROJ=$(realpath "$FILE")
        REGEX="0x[a-zA-Z0-9]{64}"

        pushd "$DIR/.." > /dev/null
        CONTRACT_ID=$(cargo r -p forc --release -- contract-id --path $CONTRACT_ARGS 2> /dev/null | $grep -oP "$REGEX")
        popd > /dev/null

        # if error print error and quit
        if [ $? -eq 0 ]; then
            echo "$CONTRACT_ID"
        else
            >&2 echo "cargo r -p forc --release -- contract-id --path $CONTRACT_ARGS"
            cargo r -p forc --release -- contract-id --path $CONTRACT_ARGS
            exit
        fi
    fi
}

get_new_predicate_id() {
    line="$1"
    PARTS=($(echo $line | $sed 's/:/ /g'))
    FILE=${PARTS[0]}
    LINE=${PARTS[1]}

    >&2 echo -e "${BOLD_WHITE}$FILE${NC}"

    SED_COMMAND="${LINE}"'!d'
    CONTRACT_ARGS=($($sed "$SED_COMMAND" $FILE))
    CONTRACT_ARGS=($(return_only_after_item "AUTO-PREDICATE-ID" "${CONTRACT_ARGS[@]}"))

    PROJ_NAME="${CONTRACT_ARGS[0]}"
    CONTRACT_ARGS=("${CONTRACT_ARGS[@]:1}")

    CONTRACT_ARGS=$(join_by " " ${CONTRACT_ARGS[@]})

    if [[ $CONTRACT_ARGS ]]; then
        PROJ=$(realpath "$FILE")
        REGEX="\[$PROJ_NAME\]: \K0x[a-zA-Z0-9]{64}"
        CONTRACT_ID=$(cargo r -p forc --release -- build --path $CONTRACT_ARGS 2> /dev/null | $grep -oP "$REGEX")
        # if error print error and quit
        if [ $? -eq 0 ]; then
            echo "$CONTRACT_ID"
        else
            cargo r -p forc --release -- build --path $CONTRACT_ARGS
            exit
        fi
    fi
}

update_line() {
    line="$1"
    PARTS=($(echo $line | $sed 's/:/ /g'))
    FILE=${PARTS[0]}
    LINE=${PARTS[1]}

    CONTRACT_ID="$2"
    if [[ $CONTRACT_ID ]]; then
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
                echo -e "    ${BOLD_RED}Aborting${NC} This contract/predicate id needs update, but git state is not clean. commit, restore first or run with \"-i\"."
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
        # cargo r -p forc --release -- contract-id --path $CONTRACT_ARGS
    fi
}

# Update require-contract-deployment tests
$grep --include \*.sw -Hno "// AUTO-CONTRACT-ID" . -R | while read line ; do
    NEW_CONTRACT_ID=$(get_new_contract_id $line)
    update_line "$line" "$NEW_CONTRACT_ID"
done

# Update predicates
root="test/src/sdk-harness/test_projects/auth"
$grep --include \*.rs -Hno "// AUTO-PREDICATE-ID" "$root" -R | while read line ; do
    NEW_PREDICATE_ID=$(get_new_predicate_id $line)
    update_line "$line" "$NEW_PREDICATE_ID"
done
