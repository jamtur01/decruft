# Dependent Type Theory

This fixture verifies that Verso-style Lean command and output fragments are merged into a single fenced code block. It also checks that hidden hover metadata does not leak into visible code text during extraction. The paragraph is intentionally verbose so content detection remains stable and deterministic across environments. 

`` `Nat : Type`[#check](#) Nat ``

```
Nat : Type
```

`` `Bool : Type`[#check](#) Bool ``

```
Bool : Type
```

`` `Nat → Bool : Type`[#check](#) Nat → Bool ``

```
Nat → Bool : Type
```

Text after the example ensures that downstream markdown rendering keeps non-code prose separate from the merged block output.