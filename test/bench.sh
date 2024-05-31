#! /bin/bash

BOLD_RED='\033[1;31m'
BOLD_GREEN="\033[1;32m"
BOLD_YELLOW='\033[1;33m'
BOLD_WHITE='\033[1;97m'
NC='\033[0m'

BRANCH="master"

print_help() {
    echo "
 -n|--qty          how many commits will be used
 -b|--branch       which branch will be used. Defaults to \"master\"
 -p|--path         path to sway project to use
 -c|--clean        what to clean. Options:
     \"results\" - will clean all csv, txt files
     \"exe\"     - will clean all compiled executables
     \"all\"     - all the above
 -o|--open         open the final report using the default browser";
}

if [[ "$#" -eq 0 ]]; then
    print_help;
    exit 1;
fi

while [[ "$#" -gt 0 ]]; do
    case $1 in
        -n|--qty) QUANTITY="$2"; shift ;;
        -b|--branch) BRANCH="$2"; shift ;;
        -p|--path) PROJ_PATH="$2"; shift ;;
        -c|--clean) CLEAN="$2"; shift ;;
        -o|--open) OPEN="1"; shift ;;
        -h|--help) print_help; exit 0;;
        *) print_help; exit 1 ;;
    esac
    shift
done

# create cache folder
CACHE="$HOME/.cache/sway-bench"
mkdir "$CACHE" -p

LOG_FILE="$CACHE/log.txt"
echo "" > "$LOG_FILE"

# exec 19> "$LOG_FILE"
# BASH_XTRACEFD="19"
# set -o xtrace

ask_confirmation() {
    echo -e "${BOLD_WHITE}Command below needs confirmation before running.${NC} $2"
    echo "> $1"
    read -p "Run the command above? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]
    then
        bash -c "$1"
    else
        exit 1
    fi
}

# clean results and exit
if [[ $CLEAN = "results" || $CLEAN = "all" ]]; then
    ask_confirmation "find \"$CACHE\" -name \"*.csv\" -delete" "It will delete all csv files in the \"$CACHE\" folder."
    ask_confirmation "find \"$CACHE\" -name \"*.txt\" -delete" "It will delete all txt files in the \"$CACHE\" folder."
    ask_confirmation "rm \"$CACHE/index.html\" -f" "It will delete \"index.html\" in the \"$CACHE\" folder."
    if [[ $CLEAN = "results" ]]; then
        exit 0
    fi
fi

if [[ $CLEAN = "exe" || $CLEAN = "all" ]]; then
    ask_confirmation "find \"$CACHE\" -type f -executable -delete" "It will delete all executable in the \"$CACHE\" folder."
    if [[ $CLEAN = "exe" ]]; then
        exit 0
    fi
fi

if [[ $CLEAN = "all" ]]; then
    exit 0
fi

PROJ_NAME="$(basename $PROJ_PATH)"

# install dependencies
if ! command -v hyperfine &>> "$LOG_FILE"
then
    ask_confirmation "cargo install hyperfine" "It will install \"hyperfine\" using cargo. "
fi


# checkout specified branch
if [ -n "$BRANCH" ]; then
    git checkout "$BRANCH" &>> "$LOG_FILE"
fi

if [ -n "$QUANTITY" ]; then
    # For each commit
    for HASH in `git log --format="%H" -$QUANTITY --reverse`; do
        # checkout this commit
        # if the repo is dirty, stash and restore after

        git checkout $HASH &>> "$LOG_FILE"
        
        BRANCH_FILENAME=$(echo $BRANCH | sed 's/[\/\\-]/_/g')

        COMMIT="$(git show -s --format='%as-%ct-%H' HEAD)"
        COMMIT="$BRANCH_FILENAME-$COMMIT"

        COMMIT_HASH="$(git show -s --format='%H' HEAD | head -c 4)"
        COMMIT_MSG=$(git log --oneline --format=%B -n 1 $HASH | head -n 1 | head -c 80)

        echo -e -n "${BOLD_WHITE}$BRANCH_FILENAME${NC} $COMMIT_HASH \"$COMMIT_MSG\""

        # compile this version if needed
        if [ ! -f "$CACHE/$COMMIT" ]; then
            echo -e -n " [compiling]"
            cargo b --release &>> "$LOG_FILE"
            cp target/release/forc "$CACHE/$COMMIT" &>> "$LOG_FILE"
        fi

        # run test if needed
        if [ ! -f "$CACHE/$COMMIT-$PROJ_NAME.csv" ]; then
            echo -e -n " [benchmark]"
            hyperfine -n "$COMMIT-$PROJ_NAME" --export-csv "$CACHE/$COMMIT-$PROJ_NAME.csv" "$CACHE/$COMMIT build -p $PROJ_PATH --release" &>> "$LOG_FILE"
        fi

        # get binary size if needed
        if [ ! -f "$CACHE/$COMMIT-$PROJ_NAME-size.txt" ]; then
            echo -e -n " [bin size]"
            rm "$PROJ_PATH/out" -rf &>> "$LOG_FILE"
            bash -c "$CACHE/$COMMIT build -p $PROJ_PATH --release" &>> "$LOG_FILE"
            stat --printf="%s" "$PROJ_PATH/out/release/$PROJ_NAME.bin" > "$CACHE/$COMMIT-$PROJ_NAME-size.txt"
        fi

        echo -e " ${BOLD_GREEN}ok${NC}"
    done
fi

# generate final report

pushd "$CACHE" &>> "$LOG_FILE"
rm index.html &>> "$LOG_FILE"
touch index.html &>> "$LOG_FILE"
echo '<!DOCTYPE html>
<html>
    <head>
        <title>Pivot Demo</title>
        <script type="text/javascript" src="https://cdnjs.cloudflare.com/ajax/libs/jquery/1.11.2/jquery.min.js"></script>
        <script type="text/javascript" src="https://cdnjs.cloudflare.com/ajax/libs/jqueryui/1.11.4/jquery-ui.min.js"></script>
        
        <link rel="stylesheet" type="text/css" href="https://cdnjs.cloudflare.com/ajax/libs/pivottable/2.23.0/pivot.min.css">
        <script type="text/javascript" src="https://cdnjs.cloudflare.com/ajax/libs/pivottable/2.23.0/pivot.min.js"></script>

        <script type="text/javascript" src="https://cdnjs.cloudflare.com/ajax/libs/d3/3.5.5/d3.min.js"></script>
        <script type="text/javascript" src="https://cdnjs.cloudflare.com/ajax/libs/pivottable/2.23.0/d3_renderers.min.js"></script>

        <script type="text/javascript" src="https://cdnjs.cloudflare.com/ajax/libs/c3/0.4.11/c3.min.js"></script>
        <script type="text/javascript" src="https://cdnjs.cloudflare.com/ajax/libs/pivottable/2.23.0/c3_renderers.min.js"></script>

        <script src="https://cdn.plot.ly/plotly-basic-latest.min.js"></script>
        <script src="https://cdnjs.cloudflare.com/ajax/libs/pivottable/2.23.0/plotly_renderers.min.js"></script>
        
        <style>
            body {font-family: Verdana;}
            .node {
              border: solid 1px white;
              font: 10px sans-serif;
              line-height: 12px;
              overflow: hidden;
              position: absolute;
              text-indent: 2px;
            }
        </style>
        <script type="text/javascript" src="https://cdnjs.cloudflare.com/ajax/libs/jqueryui-touch-punch/0.2.3/jquery.ui.touch-punch.min.js"></script>
    </head>
    <body>
        <script type="text/javascript">        
        $(function(){
            var renderers = $.extend(
                $.pivotUtilities.renderers,
                $.pivotUtilities.c3_renderers,
                $.pivotUtilities.d3_renderers,
                $.pivotUtilities.plotly_renderers
            );
            $("#output").pivotUI($("#input"), {
                renderers,
                cols: ["Msg"],
                rows: ["Branch", "Proj Name"],
                aggregatorName: "Average",
                vals: ["Comp Time Mean"],
                rendererName: "Line Chart",
            });
        });
        </script>
        <div id="output" style="margin: 30px;"></div>
        <br />
        <h3>Input table:</h3>
<table id="input" border="1" style="width: 100%">
<thead>
    <tr>
        <th>Branch</th>
        <th>Proj Name</th>
        <th>Timestamp</th>
        <th>Hash</th>
        <th>Msg</th>
        <th>Comp Time Mean</th>
        <th>Bin Size</th>
    </tr>
</thead>
<tbody>' >> index.html

for csv in *.csv; do
    BRANCH=$(echo $csv|sed 's/\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)\.csv/\1/')
    YEAR=$(echo $csv|sed 's/\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)\.csv/\2/')
    MONTH=$(echo $csv|sed 's/\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)\.csv/\3/')
    DAY=$(echo $csv|sed 's/\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)\.csv/\4/')
    TIMESTAMP=$(echo $csv|sed 's/\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)\.csv/\5/')
    HASH=$(echo $csv|sed 's/\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)\.csv/\6/')
    PROJ_NAME=$(echo $csv|sed 's/\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)-\([^-]*\)\.csv/\7/')

    popd &>> "$LOG_FILE"
    COMMIT_MSG=$(git log --oneline --format=%B -n 1 $HASH | head -n 1 | sed 's/#\([0-9]*\)/<a href="https\:\/\/github.com\/FuelLabs\/sway\/issues\/\1">#\1<\/a>/')
    pushd "$CACHE" &>> "$LOG_FILE"

    BIN_SIZE=$(cat "$CACHE/$BRANCH-$YEAR-$MONTH-$DAY-$TIMESTAMP-$HASH-$PROJ_NAME-size.txt")
    
    # fields
    # command,mean,stddev,median,user,system,min,max
    FIELD_COMMAND=$(tail -n 1 $csv | cut -d, -f1)
    FIELD_MEAN=$(tail -n 1 $csv | cut -d, -f2)
    FIELD_STDDEV=$(tail -n 1 $csv | cut -d, -f3)
    FIELD_MEDIAN=$(tail -n 1 $csv | cut -d, -f4)
    FIELD_USER=$(tail -n 1 $csv | cut -d, -f5)
    FIELD_SYSTEM=$(tail -n 1 $csv | cut -d, -f6)
    FIELD_MIN=$(tail -n 1 $csv | cut -d, -f7)
    FIELD_MAX=$(tail -n 1 $csv | cut -d, -f8)
    echo "<tr>" >> index.html
    echo "  <td>$BRANCH</td>" >> index.html
    echo "  <td>$PROJ_NAME</td>" >> index.html
    echo "  <td>$TIMESTAMP</td>" >> index.html
    echo "  <td><a href=\"https://github.com/FuelLabs/sway/commit/$HASH\">$TIMESTAMP-$HASH</a></td>" >> index.html
    echo "  <td>$TIMESTAMP-$COMMIT_MSG</td>" >> index.html
    echo "  <td>$FIELD_MEAN</td>" >> index.html
    echo "  <td>$BIN_SIZE</td>" >> index.html
    echo "</tr>" >> index.html
done

echo '</tbody>
</table>
</body>
</html>' >> index.html
popd &>> "$LOG_FILE"

# Return to master
git checkout master &>> "$LOG_FILE"

echo -e "${BOLD_YELLOW}Warning${NC}: Cache at \"${CACHE}\" is using $(du $CACHE -BM | cut -f1) of your disk space"

if [ -n "$OPEN" ]; then
    if which xdg-open &>> "$LOG_FILE"
    then
        xdg-open "$CACHE/index.html"
        sleep 1
    elif which gnome-open &>> "$LOG_FILE"
    then
        gnome-open "$CACHE/index.html"
        sleep 1
    fi
fi
