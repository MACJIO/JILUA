# JILUA

This is fast LUA JIT decompiler wrote on Rust.

Work in progress.

Currently, first part of IR is done.

It's an example of decompiler output.

```
Block(0)
    v0 = _G["lib"]
    v0 = v0["sys"]
    v0 = v0["exec"]
    v1 = "cat /etc/version"
    v0 = v0(v1)
    v1 = v0
    v0 = v0["gsub"]
    v2 = "\n"
    v3 = ""
    v0 = v0(v1, v2, v3)
    if v0
Block(12)
    v0 = ""
Block(13)
    v1 = "\'"
    v2 = v0
    v3 = "\'"
    v1 = v1 ~ v2 ~ v3
    return v1
```

But still, I need to add loop detection, more analysis and make decompiler interaction interface.
 