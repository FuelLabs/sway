IFS=',' read -ra PACKAGES <<< "$PACKAGE_NAMES"
          mismatch=0
          for PACKAGE in "${PACKAGES[@]}"; do
            VERSION_FIRST=$(toml get ./Cargo.toml "workspace.dependencies.$PACKAGE.version")
            VERSION_SECOND=$(toml get ./test/src/sdk-harness/Cargo.toml "dependencies.$PACKAGE.version")
            echo "$PACKAGE Version - First: $VERSION_FIRST, Second: $VERSION_SECOND"
            if [ "$VERSION_FIRST" != "$VERSION_SECOND" ]; then
              echo "Version mismatch for $PACKAGE: First: $VERSION_FIRST, Second: $VERSION_SECOND"
              mismatch=1
            fi
          done
          if [ $mismatch -ne 0 ]; then
            echo "Version mismatch between fuel-core-client used in sdk-harness and rest of sway repo.
	    This will cause problems if two versions are incompatible or it might simply cause invalid/outdated test suite.
            If you are bumping fuel-core versions used in sway repo, please also use same version in sdk-harness."
            exit 1
          else
            echo "All specified package versions match."
	  fi	
