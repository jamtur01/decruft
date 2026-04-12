I am bridging python and C++, namely calling a python function inside C++ host model. I got an error that I failed to find any clue in the web. Please help! Any insight would be deeply appreciated!

The error looks like:

"

```
150: Fatal Python error: _Py_GetConfig: the function must be called with the GIL held, after Python initialization and before Python finalization, but the GIL is released (the current Python thread state is NULL)
150: Python runtime state: finalizing (tstate=0x000014e69ea28550)
150: 
150: 
150: Program received signal SIGABRT: Process abort signal. 
```

"

Are the results produced before exit reliable, even though the model cast the GIL issue at exit?

My code is too large to be posted here. Actually, I am porting a python module into a global climate model written in C++. The idea is :

```
for (...)  loop #1
   for (...)  loop #2
     call python function(scalar1, scalar2, scalar3...)
     pass tendencies obtained from python back to the C++ model
```

Many thanks!

I am using python/3.13-26.1.0