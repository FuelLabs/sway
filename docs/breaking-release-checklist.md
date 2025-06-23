# Breaking Release Checklist

- [ ] Ensure that the `forc migrate` tool **in the latest patch release before the breaking change release** contains all migration steps.
- [ ] Promote experimental features to standard features:
  - [ ] Remove feature flags and all conditional code from the compiler.
  - [ ] Remove experimental `cfg` attributes from all the Sway codebase in the `sway` repository. (E.g., `std` library, E2E tests, in-language tests, etc.)
  - [ ] Remove testing experimental features from E2E tests by deleting all the `test.<feature>.toml` files.
  - [ ] Remove testing experimental features from `ci.yml`.
  - [ ] Close the GitHub tracking issues.
- [ ] In the breaking change release, unregister all the migration steps in the `forc migrate`. (Do not delete the migrations. We want to keep them as examples for similar future migrations and for learning purposes.)
- [ ] Update documentation;
  - [ ] Ensure that the experimental feature itself is fully documented.
  - [ ] Ensure that all _Notes_ describing the feature as experimental are removed.
  - [ ] Ensure that all related documentation is updated.
