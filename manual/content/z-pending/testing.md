+++
title = "testing.md"
weight = 340
sort_by = "weight"
+++
TODO:
* tokenize tool
* common/parsing-tests/refresh
* syntoniq/test-data/actual/
* how to play test data
  * csound a.csd
  * timidity a.mts.midi
  * for mpe.midi:
    * start surge XT
    * use `aplaymidi -l` to find suitable port
    * `aplaymidi --port 'Midi Through' file.mpe.midi`

* test coverage requirements including rationale for 100% in some files

100% coverage:

* common/parsing/pass1
* common/parsing/pass2
* in common/parsing/
  * layout.rs
  * model.rs
  * pass*.rs
  * score.rs
  * timeline.rs
* in common/parsing/score/
  * generator.rs

as this ensures all error conditions are tested and that there are no unreachable code paths. Unreachable code paths would indicate that the parser isn't coded as tightly as it should be. These would arise if later code relies on earlier validations, which is fragile.
