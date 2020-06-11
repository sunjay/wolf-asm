- expand include paths
- collect consts and verify that their names are unique
- substitute values for const names
- validate that label names are unique
- validate that there is exactly 1 of each section and that they are in the
  right order
- validate that `.zero` and `.uninit` have a non-negative argument
- note: `.bytes` is allowed to take an empty string
- instructions have valid names and the expected number/type of arguments

At some point:

- generate offsets for each label by counting up the bytes before each label
- substitute values for label offsets
- actually generate the bytes for each instruction

Passes:

1. Recursive include expansion
  - open and parse the files specified in `.include` directives
  - place the parsed statements from those files inline
  - up to a recursion limit of 1000
2. const expansion, validation, and instruction checking
  - at the end of this the only remaining names must be labels
3. label offset generation
4. label expansion
  - name resolution error where label names could not be resolved
  - after this step, there should be no more identifiers in the body of
    any instruction or directive
5. executable generation
