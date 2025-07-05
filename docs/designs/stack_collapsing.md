# Sidebar Stack Collapsing

Scratchpad for working out the indexing for stack even/odd height and collapse vs. no collapse.

## Odd + No Collapse
rows: 13
bottom-y: 13 (rows)
```
╦═══════╗
║ Stack ║
╟───┬───╢
║   │   ║
╟───┼───╢
║   │   ║
╟───┼───╢
║   │   ║
╟───┼───╢
║   │   ║
╟───┼───╢
║   │   ║
╟───┼───╢
║   │   ║
╫───┴───╢
```

## Even + No Collapse
rows: 12
bottom-y: 11 (rows-1)
```
╦═══════╗
║ Stack ║
╟───┬───╢
║   │   ║
╟───┼───╢
║   │   ║
╟───┼───╢
║   │   ║
╟───┼───╢
║   │   ║
╟───┼───╢
║   │   ║
╟───┴───╢
╫───────╢
```

## Odd + Collapse
rows: 13
bottom-y: 13 (rows)
skipped-y: 11 (rows-2)
start-y: 9 (rows-4)
```
╦═══════╗
║ Stack ║
╟───┬───╢
║   │   ║
╟───┼───╢
║   │   ║
╟───┼───╢
║   │   ║
╟───┼───╢
║   │   ║
╟───┴───╢
║[120  ]║
╟───┬───╢
║   │   ║
╫───┴───╢
```

## Even + Collapse
rows: 12
bottom-y: 12 (rows)
skipped-y: 10 (rows-2)
start-y: 7 (rows-5)
```
╦═══════╗
║ Stack ║
╟───┬───╢
║   │   ║
╟───┼───╢
║   │   ║
╟───┼───╢
║   │   ║
╟───┴───╢
╟───────╢
║[120  ]║
╟───┬───╢
║   │   ║
╫───┴───╢
```