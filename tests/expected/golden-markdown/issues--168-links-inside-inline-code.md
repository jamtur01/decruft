# Inline Code with Links

Documentation tools sometimes wrap identifiers in links inside inline code. For example, the type `[Nat](https://example.org/doc/ref/Nat)` is a common type in the language. 

Multiple links can appear in a single code span: `[List](https://example.org/doc/ref/List) [Nat](https://example.org/doc/ref/Nat)` should render as `List Nat` without any markdown link syntax. 

Regular links outside of code should still work normally. See [the guide](https://example.org/guide) for more details. This paragraph ensures there is enough content for stable extraction.