# Update

## SYNOPSIS

`forc update` [_options_]

## DESCRIPTION

Updates each of the dependencies so that they point to the latest suitable commit or version given their dependency declaration. The result is written to the `Forc.lock` file.

## OPTIONS

`-c`, `--check`

Checks if the dependencies have newer versions. Won't actually
perform the update, will output which ones are up-to-date and
outdated

`-d` _target_dependency_

Dependency to be updated. If not set, all dependencies will be
updated

`-p`, `--path` _path_

Path to the project, if not specified, current working directory
will be used
