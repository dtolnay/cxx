Important changes:

- `no-abort` replacing panic with exception handling.
- `require lifetime annotations on all returned references` - `no-abort` dependency

Misc changes:
- `__WORKERD_CXX__ define to identify when workerd-cxx fork is in use` crate names are still
    `cxx*`, so extra mechanism is needed to identify the presence of the fork.
- `build fixes` to better suit our build environment. These might be obsolete since there seems to 
    better rules_rust support upstream.
- `site fixes` fixing CI not to build a site and to run periodically
