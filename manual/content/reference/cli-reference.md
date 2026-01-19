+++
title = "Command Line Reference"
weight = 10
sort_by = "weight"
+++

The `syntoniq` and `syntoniq-kbd` commands have detailed `--help` options that explain how to invoke them from the command line. Both commands also support completion for a number of common shells. Run `syntoniq completion --help` or `syntoniq-kbd completion --help` for details.

The most important commands are `syntoniq-kbd run` and `syntoniq generate`. You can run `syntoniq-kbd run --help` and `syntoniq generate --help` for help on those.

# Example Commands

The following command converts a score to all available outputs.
```sh
syntoniq generate \
   --score=score.stq \
   --csound=score.csd \
   --midi-mpe=score-mpe.midi \
   --midi-mts=score-mts.midi \
   --json=score-timeline.json
```

There are several command-line examples in the [Complete Example](../../microtonality/example/) section. The [Keyboard](../../keyboard/) chapters include examples of running the keyboard.
