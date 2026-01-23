+++
title = "Editor Support"
weight = 40
sort_by = "weight"
+++

Syntoniq language files are just text files. They must contain only valid UTF-8 encoding and can be edited in any modern text editor.

# Eventual Goal

FUTURE: update if we implement LSP or the formatter.

We hope to provide an [LSP (Language Server Protocol)](https://microsoft.github.io/language-server-protocol/) server for Syntoniq. The Syntoniq parser was written with this in mind. This will provide semantically aware syntax highlighting and other features. We also hope to provide an automatic formatter that can do things align notes and dynamics in score blocks by temporal position, format/align directive parameters, and align notes in layout and scale definitions. These features are not planned for version 1.0 and will be implemented based on available time and interest.

# TextMate Syntax Highlighting

The syntax highlighting in the manual is generated using a TextMate JSON file: [syntoniq.tmLanguage.json](https://raw.githubusercontent.com/jberkenbilt/syntoniq/refs/heads/main/manual/syntaxes/syntoniq.tmLanguage.json). Some editors, such as Visual Studio Code, may support these directly.

# Emacs

I hope to someday create an emacs mode for Syntoniq since I use emacs myself. In the interim, this minimal mode, based on `generic-mode`, is better than nothing.

```elisp
;; Syntoniq placeholder until we have LSP support.
(define-generic-mode syntoniq-generic-mode
  '(?\;)
  nil
  '(
    ("\\^[0-9]+\\(|[0-9]+\\)" . font-lock-variable-name-face)
    ("[0-9]+\\(/[0-9]+\\)?" . font-lock-variable-name-face)
    ("[a-zA-Z][a-zA-Z0-9_\\*\\^/\\.|+!\\\\#%&-]*" . font-lock-string-face)
  )
  '("\\.stq\\'")
  nil
  "Generic mode for Syntoniq files."
)
(require 'markdown-mode)
(add-to-list 'markdown-code-lang-modes '("syntoniq" . syntoniq-generic-mode))
```
