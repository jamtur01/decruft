# Dependent Type Theory

This fixture verifies that Verso-style Lean command and output fragments are merged into a single fenced code block. It also checks that hidden hover metadata does not leak into visible code text during extraction. The paragraph is intentionally verbose so content detection remains stable and deterministic across environments. 

`` `Nat : Type`[#check](https://lean-lang.org/theorem_proving_in_lean4/dependent_type_theory.html#) Nat ``

```
Nat : Type
```

`` `Bool : Type`[#check](https://lean-lang.org/theorem_proving_in_lean4/dependent_type_theory.html#) Bool ``

```
Bool : Type
```

`` `Nat → Bool : Type`[#check](https://lean-lang.org/theorem_proving_in_lean4/dependent_type_theory.html#) Nat → Bool ``

```
Nat → Bool : Type
```

Text after the example ensures that downstream markdown rendering keeps non-code prose separate from the merged block output.