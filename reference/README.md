# reference

Trees phiber reads and checks itself against. None is built.

- `blender/`: Blender's source as a git submodule, pinned to the v5.2.4 release tag (the latest release when added). Cloned with Git LFS content skipped, so binary assets are pointer files; the C and C++ source is complete.

phiphx (`crates/phiphx` in phikit), phiber's asset core, checks its `.blend` reader against this tree.
Update with: `git -C reference/blender fetch --depth 1 origin tag <vX.Y.Z>` then check out the tag and commit the new pointer.
