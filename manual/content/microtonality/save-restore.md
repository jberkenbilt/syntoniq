+++
title = "Save and Restore Pitch"
weight = 60
sort_by = "weight"
+++

Now that you've learned how to do transposition in Syntoniq, we'll introduce three new directives. Sometimes transposition can be disorienting in Syntoniq as it lends itself to lots of use of relative pitches. These directives provide an alternative to the `transpose` directive and can help you stay oriented.

We will cover three directives:
* `save_pitch` — saves the pitch of a specified note in a *variable*
* `restore_pitch` — transposes a part so that the pitch stored in a variable is assigned to a particular note
* `check_pitch` — checks that all its parameters, which can be notes, variables, or pitches, have the same pitch value

You can implement `transpose` using `save_pitch` and `restore_pitch`. `check_pitch` doesn't produce or change any musical output. It's sole purpose is to generate an error if things aren't how you think they are.

TODO HERE AFTER PIVOT

You have now seen how to use transposition in Syntoniq, and you've seen most of the important features. The rest is simple in comparison. The remaining features of the language will be covered in the next part.
