# Get the version of a package from a Cargo.toml file.
# The version can be specified in two ways:
# 1. some_package = { version = "x.y.z", ... }
# 2. some_package = "x.y.z"
get_version() {
	local file="$1"
	local key="$2"
	local version
	version=$(toml get "$file" "$key.version" 2>/dev/null)
	if [ -z "$version" ]; then
		version=$(toml get "$file" "$key" 2>/dev/null)
	fi
	echo "$version"
}

IFS=',' read -ra PACKAGES <<< "$PACKAGE_NAMES"
mismatch=0
for PACKAGE in "${PACKAGES[@]}"; do
	VERSION_FIRST=$(get_version ./Cargo.toml "workspace.dependencies.$PACKAGE")
	VERSION_SECOND=$(get_version ./test/src/sdk-harness/Cargo.toml "dependencies.$PACKAGE")
        printf "$PACKAGE\n    sway repo:   $VERSION_FIRST\n    sdk-harness: $VERSION_SECOND\n"
        if [ "$VERSION_FIRST" != "$VERSION_SECOND" ]; then
        	printf "ERROR: Version mismatch for $PACKAGE\n"
        	mismatch=1
        fi
done
if [ $mismatch -ne 0 ]; then
	printf "\nVersion mismatch between dependencies used in the sdk-harness tests and the rest of the sway repository.\nThis will cause problems if two versions are incompatible or it might simply cause invalid/outdated test suite.\nIf you are bumping dependency versions used in the sway repo, please use the same versions in the sdk-harness.\n"
	exit 1
else
	printf "\nAll specified package versions match.\n"
fi
