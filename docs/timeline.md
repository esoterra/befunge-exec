# Timeline Design

## Human-readable log format

```
0 (0,0) > push(0)
> (1,0) >
1 (2,0) > push(1)
+ (3,0) > pop(1) pop(0) push(1)
v (4,0) v
< (4,1) <
^ (1,1) ^
> (1,0) >
1 (2,0) > push(1)
+ (3,0) > pop(1) pop(1) push(2)
```

## Ascii UI

```
║   0    ║   >   ║   1    ║     +     ║
║  zero  ║ right ║  one   ║    Add    ║
║ (0,0)  ║ (1,0) ║ (2,0)  ║   (3,0)   ║
║ []→[0] ║       ║ []→[1] ║ [0,1]→[1] ║
```