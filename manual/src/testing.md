TODO:
* tokenize tool
* common/parsing-tests/refresh
* syntoniq/test-data/actual/
* how to play test data
* test coverage requirements including rationale for 100% in some files

100% coverage:

* common/parsing/pass1
* common/parsing/pass2
* in common/parsing/
  * layout.rs
  * model.rs
  * pass*.rs
  * score.rs
  * score/generator.rs
  * timeline.rs

as this ensures all error conditions are tested and that there are no unreachable code paths. Unreachable code paths would indicate that the parser isn't coded as tightly as it should be. These would arise if later code relies on earlier validations, which is fragile.
