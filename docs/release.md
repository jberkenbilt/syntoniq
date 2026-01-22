# Prepare Release

Remember: the version in Cargo.toml is the *next* version to release.

Create docs/github-releases/vX.Y.Z.md with release notes. Copy the previous one and modify it for consistency. Push to main.

Make sure the release date is set in manual/content/appendices/release-notes.md

```sh
git commit -a -m"Prepare version $version"
```

Push to main.

# Create Release

Common setup:
```sh
version=$(toml-to-json < Cargo.toml | jq -r .workspace.package.version)
repo=~/source/syntoniq
release=~/Q/storage/releases/syntoniq/$version
mkdir -p $release
```

Tag the release. Do this *after* CI is built in case there is a problem. We don't trigger anything on the tag. In the source repository:

```sh
cd $repo
git tag -s v$version -m"Syntoniq version $version"
git push syntoniq v$version
```

Download the distribution from CI. In release archive directory:

```sh
cd $release
unzip ~/Downloads/distribution.zip
patmv s/syntoniq/syntoniq-$version/ *
\rm -f *.sha256
files=(*)
sha256sum ${files[*]} >| syntoniq-$version.sha256
gpg --clearsign --armor syntoniq-$version.sha256
mv syntoniq-$version.sha256.asc syntoniq-$version.sha256
cosign sign-blob syntoniq-$version.sha256 --bundle syntoniq-$version.sha256.sigstore
chmod 444 *
```

Create the actual release.

```sh
cd $repo
gh release create v$version --title "Syntoniq version $version" -F docs/github-releases/v$version.md $release/*
```
