### RSA example

Here is the key generation code:

```
1p = 61
 2q = 97
 3
 4print(f"n={p*q}")
 5# n=5917
 6
 7phi = (p-1)*(q-1)
 8
 9print(f"phi={phi}")
10# phi=5760
```

This gives us the public and private keys.