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

if ! command -v toml >/dev/null 2>&1; then
	printf "ERROR: toml-cli is required to check SDK dependency versions.\n" >&2
	exit 1
fi

IFS=',' read -ra PACKAGES <<< "$PACKAGE_NAMES"
mismatch=0
for PACKAGE in "${PACKAGES[@]}"; do
	VERSION_FIRST=$(get_version ./Cargo.toml "workspace.dependencies.$PACKAGE")
	VERSION_SECOND=$(get_version ./test/src/sdk-harness/Cargo.toml "dependencies.$PACKAGE")
	if [ -z "$VERSION_FIRST" ] || [ -z "$VERSION_SECOND" ]; then
		printf "ERROR: Could not read %s from both workspace and SDK harness manifests.\n" "$PACKAGE" >&2
		mismatch=1
	fi
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

# The public cargo-generate templates must use the same Rust SDK generation as
# the SDK harness tested above. Keep this list explicit so a new program-type
# template must opt into the compatibility check.
FUELS_VERSION=$(get_version ./Cargo.toml "workspace.dependencies.fuels")
TEMPLATES=(
	"./templates/sway-predicate-test-rs/template/Cargo.toml"
	"./templates/sway-script-test-rs/template/Cargo.toml"
	"./templates/sway-test-rs/template/Cargo.toml"
)

template_mismatch=0
mapfile -t DISCOVERED_TEMPLATES < <(
	find ./templates -mindepth 3 -maxdepth 3 -type f \
		-path '*/template/Cargo.toml' | sort
)
if [ "${DISCOVERED_TEMPLATES[*]}" != "${TEMPLATES[*]}" ]; then
	printf "ERROR: Public template manifest list is out of sync.\n" >&2
	printf "Expected: %s\n" "${TEMPLATES[*]}" >&2
	printf "Found:    %s\n" "${DISCOVERED_TEMPLATES[*]}" >&2
	template_mismatch=1
fi

for TEMPLATE in "${TEMPLATES[@]}"; do
	TEMPLATE_VERSION=$(get_version "$TEMPLATE" "dev-dependencies.fuels")
	if [ -z "$FUELS_VERSION" ] || [ -z "$TEMPLATE_VERSION" ]; then
		printf "ERROR: Could not read fuels version from workspace or %s\n" "$TEMPLATE" >&2
		template_mismatch=1
	fi
	printf "fuels\n    sway repo: $FUELS_VERSION\n    $TEMPLATE: $TEMPLATE_VERSION\n"
	if [ "$FUELS_VERSION" != "$TEMPLATE_VERSION" ]; then
		printf "ERROR: Rust SDK version mismatch for %s\n" "$TEMPLATE"
		template_mismatch=1
	fi
done

if [ $template_mismatch -ne 0 ]; then
	printf "\nRust SDK versions in the public templates must match workspace.dependencies.fuels.\n"
	exit 1
else
	printf "\nAll public template Rust SDK versions match the Sway SDK harness.\n"
fi
