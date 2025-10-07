#! /bin/bash

PROJ=$1
SLEEP_BETWEEN=$3
CACHE="$HOME/.cache/sway-bench"

NC='\033[0m'
BOLD_GREEN="\033[1;32m"
BOLD_RED='\033[1;31m'
BOLD_WHITE='\033[1;97m'

# $1 = commit hash
compile_and_cp_to_cache() {
    if [ ! -f "$CACHE/$1" ]; then
        if [[ -n $SLEEP_BETWEEN ]]; then
            sleep "$SLEEP_BETWEEN"
        fi
        cargo b --release &>> /dev/null
        cp target/release/forc "$CACHE/$1" &>> /dev/null
    fi
}

run_cargo() {
    if [ "$2" = "" ]; then
        bash -c "$CACHE/$CACHENAME build --path $PROJ" &>> /dev/null
        echo "$?"
    else
        bash -c "$CACHE/$CACHENAME $2 --path $PROJ" &>> /dev/null
        echo "$?"
    fi
}

INITIAL_COMMIT="$(git show -s --format='%H' HEAD)"
END_COMMIT=""

echo "Forc command will be:"
if [ "$2" = "" ]; then
    echo "> forc build --path $PROJ"
else
    echo "> forc $2 --path $PROJ"
fi

echo "Starting the search at: "
echo -n "    "
git log -1 --oneline

echo -n "Running: "

CACHENAME="$(git show -s --format='%as-%ct-%H' HEAD)"
compile_and_cp_to_cache "$CACHENAME"
INITIAL_COMPILATION_STATUS=$(run_cargo "$1" "$2")

if [ "$INITIAL_COMPILATION_STATUS" = "0" ]; then
    echo -e " ${BOLD_GREEN}Ok${NC}"
    echo ""
    echo "Searching the newest version which compilation was failing."
else
    echo -e " ${BOLD_RED}Failed${NC}"
    echo ""
    echo "Searching the newest version which compilation was succeeding."
fi


for HASH in `git log --format="%H" --tags --no-walk`; do
    git checkout "$HASH" &>> /dev/null
    git checkout . &>> /dev/null

    git log -1 --oneline

    CACHENAME="$(git show -s --format='%as-%ct-%H' HEAD)"
    compile_and_cp_to_cache "$CACHENAME"
    LAST_STATUS=$(run_cargo "$1" "$2")
    if [ "$INITIAL_COMPILATION_STATUS" != "$LAST_STATUS" ]; then
        echo -e "^^^^^^^^^ ${BOLD_WHITE}This version result is different!${NC}"
        break
    fi
done

END_COMMIT="$(git show -s --format='%H' HEAD)"

echo ""
echo -e "${BOLD_WHITE}Starting bisect between: ${NC}$INITIAL_COMMIT..$END_COMMIT"

git checkout $INITIAL_COMMIT &>> /dev/null

git bisect start &>> /dev/null
git bisect new $INITIAL_COMMIT &>> /dev/null
git bisect old $END_COMMIT &>> /dev/null

while :
do
    #echo "-----------------------"
    #git --no-pager bisect visualize --oneline

    git bisect next | grep "is the first new commit" &>> /dev/null
    if [ "$?" = "0" ]; then
        FIRST_COMMIT="$(git bisect next 2>&1 | head -n 1 | cut -f1 -d" ")"
        git checkout "$FIRST_COMMIT" &>> /dev/null
        break
    fi

    git bisect next | grep "Bisecting"
    if [ "$?" != "0" ]; then
        break
    fi

    git checkout . &>> /dev/null

    CACHENAME="$(git show -s --format='%as-%ct-%H' HEAD)"
    compile_and_cp_to_cache "$CACHENAME"
    LAST_STATUS=$(run_cargo "$1" "$2")
    if [ "$LAST_STATUS" = "$INITIAL_COMPILATION_STATUS" ]; then
        git bisect new &>> /dev/null
    else
        git bisect old &>> /dev/null
    fi
done

FOUND_COMMIT="$(git show -s --format='%H' HEAD)"

echo -e "${BOLD_GREEN}Found!${NC} - ${FOUND_COMMIT}"
echo ""


# check this commit has the same behaviour
echo -n "checking the found commit has the same behaviour as the initial commit..."

CACHENAME="$(git show -s --format='%as-%ct-%H' HEAD)"
compile_and_cp_to_cache "$CACHENAME"
LAST_STATUS=$(run_cargo "$1" "$2")

if [ "$INITIAL_COMPILATION_STATUS" != "$LAST_STATUS" ]; then
    echo -e " ${BOLD_RED}Unexpected exit code${NC}"
    exit 1
fi

echo -e " ${BOLD_GREEN}Ok${NC}"

## check the previous commit has the inverse
echo -n "checking the previous commit has the inverse behaviour as the initial commit..."

git checkout HEAD~1 &>> /dev/null
PREVIOUS_COMMIT="$(git show -s --format='%H' HEAD)"
CACHENAME="$(git show -s --format='%as-%ct-%H' HEAD)"
compile_and_cp_to_cache "$CACHENAME"
LAST_STATUS=$(run_cargo "$1" "$2")

if [ "$INITIAL_COMPILATION_STATUS" = "$LAST_STATUS" ]; then
    echo -e " ${BOLD_RED}Unexpected exit code${NC}"
    exit 1
fi

echo -e " ${BOLD_GREEN}Ok${NC}"

echo ""

git checkout . &>> /dev/null
git bisect reset &>> /dev/null

git checkout "$FOUND_COMMIT" &>> /dev/null
echo "This is the commit that changed the compiler behavior"
git log -1 --oneline
