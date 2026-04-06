# Adding or Modifying Directives

To add a directive:
* Add a struct to `common/src/parsing/score/directives.rs`. The documentation comments automatically get added to the manual and `syntoniq doc`.
  * There must be a `span` field
  * Other fields may have type `Spanned<T: CheckValue>`. You can also use `Option<Spanned<T>>` or `Vec<Spanned<T>>`.
  * See the `FromRawDirective` macro for additional notes, though anything for which the above is not adequate would likely require changes elsewhere in the grammar.
* Implement a `validate` method similar to the existing ones
* Add a tag to the Directive enum in the same file

You can also just add a field to an existing struct.

**If copying an existing struct, be careful to avoid cut and paste errors in the doc strings.**

From here, getting the code to compile again likely gives you the rest of the steps.
