Example Go code.

```
package main

type Person struct {
    Name string
    Age  int
}

func (p *Person) Sleep() int {
    p.Age += 1
    return p.Age
}
```