# Syntoniq Manual

See also [../docs/README.md](../docs/README.md) for internally facing documentation.

The manual uses Zola with the book theme.

* https://www.getzola.org/
* https://www.getzola.org/themes/book/
* https://github.com/getzola/book

TODO: 2026-01-19: Zola 0.22 was recently released, and it replaces Syntect with Giallo, which breaks custom syntax highlighting based on sublime-syntax files. We have to pin to Zola 0.21 until this is resolved. It should be resolved ASAP.

```sh
cargo install --locked --git https://github.com/getzola/zola
```

```
zola init
zola build
zola serve
```

See ../Taskfile.yml and static-src/Taskfile.yml for manual-related tasks.

```
cd manual/themes
git clone https://github.com/getzola/book
```

The `zola-book-vendor` branch is sitting on a commit that has a pristine checkout of the book theme. To bring in a new version of the book theme, move to that branch, sync changes, and then merge that back into main. The main branch removes unwanted files from the book theme and may contain other fixes or tweaks. When committing to the `zola-book-vendor` branch, always included detailed information about the upstream commit, e.g.

```
commit 4ee06ce568e1c9f6d19f53bf521fb267603bc6c4 (HEAD -> master, origin/master, origin/HEAD)
Author: Miguel Pimentel <contact@miguelpimentel.do>
Date:   Fri Mar 14 12:12:57 2025 -0700
```

Each section of the manual starts with a header like this:

```
+++
title = "??TITLE??"
weight = 0
sort_by = "weight"
+++
```

The `./ordering` script can be used for tweaking section order:
* git commit
* Run `./ordering --current >| /tmp/a`
* Edit /tmp/a to put the sections in order
* Run `./ordering --apply /tmp/a`

When editing templates, remember to use `get_url` for absolute paths in the Zola templates -- see manual/templates/index.html.

# Checking links

```
cargo install --locked lychee
task manual-build
lychee --remap "https://syntoniq.cc/manual/ file://$PWD/public/" public/**/*.html
```


# Elisp Help

The following registers can be useful:

```elisp
(progn
  (set-register ?s "♯")
  (set-register ?b "♭")
  (set-register ?x "𝄪")
  (set-register ?f "𝄫")
  (set-register ?u "↑")
  (set-register ?d "↓")
)
```

# Generated Content

There is a magic comment `<!-- generate ... -->` that can appear in markdown sources. It has a very exact syntax that is recognized by `./autogen`.

Generated sections are always delimited with
```
<!-- generate k=v k=v ... -->
# generated material
<!-- end-generate -->
```

Valid operations:
* `include=file checksum=...` -- include the contents of `static-src/file` verbatim. The checksum is updated if the file changes so we can avoid gratuitously updating files. This can be used to include source examples or other things. Files in `static-src` can be generated or manual. The script knows to quote .stq files with ` ```syntoniq ` and may have other special case logic.

Audio files can be automatically generated from stq files for the manual. You have to add them to `manual/static-src/Taskfile.yml`.

# Keyboard HTML files

To get a keyboard HTML file, get the keyboard in the right state, then run `curl http://localhost:8440/board` and save to a file. The keyboard HTML files in `static` were generated that way. The ones for row and column numbering were subsequently edited.
