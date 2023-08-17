# Neutron Astroport Reward Distributor

This contract holds funds in the form of vault tokens and distributes the underlying assets to an address as rewards.
The current version of this contract assumes that the reward vault base token is an Astroport LP token.

## Scripts

All scripts are contained in the `scripts` directory. Below follows short descriptions of their use and intended
purpose.

* `bpsync.sh` -
    Copies branch protection rules from one (GitHub) repo and branch to another. By default targets the repo that
    contains it, so that running it without modifications copies the rules from this repo (*apollo-template*). Run the
    script without arguments for usage instructions.
* `install-git-hooks.sh` - As the name implies, installs git hooks in the current repo. The git hooks installed are the
    `pre-commit` and `commit-msg` hooks. These hooks perform checks/modifications locally that are required to make
    checks pass on the remote, typically before merging a pull request.
* `pre-commit.sh` - The script which is copied and used as the `pre-commit` hook. Can be run manually.
* `todo-lint.sh` - Finds and reports uses of "TODO" (in varying case) in comments and elsewhere in files with associated
    file extensions (see script source code for specifics).

## GitHub Actions

Below follows short descriptions for the CI workflows included in the repository.

* _Conventional commit check_ (`cc.yml`) -
    Checks commit message headers in commit history on pull request to `master` and ensures that they adhere to the
    [Conventional Commits](https://www.conventionalcommits.org) specification.
* _Check for errors_ (`check.yml`) - Runs `cargo check` to check for errors.
* _Test Coverage_ (`coverage.yml`) -
    Checks test code coverage using [tarpaulin](https://crates.io/crates/cargo-tarpaulin).
* _Check licenses and sources_ (`licenses.yml`) -
    Runs [cargo-deny](https://crates.io/crates/cargo-deny) to check for incompatible licenses, security advisories,
    among other things
* _Linting and formatting_ (`lint-format.yml`) -
    An assortment of linting and formatting tools to ensure the code base is consistent, clean and maintainable.
* _Test Suite_ (`test.yml`) - A suite of unit and integration tests.
