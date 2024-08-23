# seda-wasm-vm

## Updating The Library Files

1. Go to the `Actions` tab on Github.
2. Click `Manual Build` on the left hand side bar.
3. Click `Run workflow`.
	1. Select the branch.
	2. Enter the CI password(note what you type is in plain text).
	3. Make sure `all` is selected.
	4. Do not click `Enable debug mode`.
4. A build will show up in the `workflow runs` section.
5. Once that has a green check mark next to it you can click on the `Manual Build` next to it.
6. At the bottom of that page it shows `Artifacts`.
	1. If you followed step 3 it should be named `branch-refs-heads-feat-proxy_http_fetch-import-arch-all-debug-false`.
7. Click the download icon next to the artifact name and it will download as a zip file.
8. Place those files in the `tallyvm` directory overwriting the files.
	1. Note this does not update the macOS one. To build that one on a macOS machine run `cargo build --release`, and copy the produced `.dylib` file to the `tallyvm` directory.