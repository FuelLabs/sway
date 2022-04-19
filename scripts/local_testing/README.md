# Local Testing
Tiny scripts for running formatting checks or tests locally. [act](https://github.com/nektos/act) is required for running all github actions.

## To run as git hooks
*Example shows running them as a pre-push hook, but can work pre-commit to, I find that to be a bit of an overkill*

Add the following in `.git/hooks/pre-push`:
```
#!/bin/bash

for hook in .git/hooks/pre-push.d/*; do
    bash $hook
    RESULT=$?
    if [ $RESULT != 0 ]; then
        echo ".git/hooks/pre-push.d/$hook returned non-zero: $RESULT, abort commit"
        exit $RESULT
    fi
done

exit 0
```

Then `mkdir .git/hooks/pre-push.d/`

Then link the script you want to run, e.g:
`ln -s $PWD/scripts/local_testing/formatting.sh .git/hooks/pre-push.d/`
`ln -s $PWD/scripts/local_testing/local_gh_actions.sh .git/hooks/pre-push.d/`