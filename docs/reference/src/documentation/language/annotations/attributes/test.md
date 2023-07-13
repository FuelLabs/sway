# Test

Sway provides the `#[test]` attribute which enables unit tests to be written in Sway.

## Success case

The `#[test]` attribute indicates that a test has passed if it did not revert.

```sway
{{#include ../../../../code/language/annotations/src/main.sw:success_test}}
```

## Revert Case

To test a case where code should revert we can use the `#[test(should_revert)]` annotation. If the test reverts then it will be reported as a passing test.

```sway
{{#include ../../../../code/language/annotations/src/main.sw:revert_test}}
```

We may specify a code to specifically test against.

```sway
{{#include ../../../../code/language/annotations/src/main.sw:revert_code_test}}
```
