# Dependent Type Theory

This fixture ensures empty Verso code blocks are preserved as blank lines when adjacent Lean command and output fragments are merged. 

`#check true`

```
Bool.true : Bool
```

`` `/- Evaluate -/

#eval 5 * 4`

```
20
```

Trailing prose ensures normal extraction continues.