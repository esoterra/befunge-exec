# Notes

```html
<div class="bft-grid" data-bft-goal="stack==[77]">
    <div class="bft-cell"></div>
    <div class="bft-cell"></div>
    <div class="bft-cell"></div>
    <div class="bft-cell"></div>
    <div class="bft-cell" data-bft-fixed="true">
        <div class="bft-tile" data-bft-value="3"></div>
    </div>
</div>

<div class="bft-inventory">
    <div class="bft-tile" data-bft-value="3"></div>
    <div class="bft-tile" data-bft-value="7"></div>
    <div class="bft-tile" data-bft-value="8"></div>
    <div class="bft-tile" data-bft-value="+"></div>
    <div class="bft-tile" data-bft-value="*"></div>
</div>
```

Stacks of tiles

```html
<div class="bft-inventory">
    <div class="bft-stack" data-bft-stack-amount="10">
        <div class="bft-tile" data-bft-value="*"></div>
    </div>
</div>
```

# BFT Classes

### Grid (`bft-grid`)

The program space

<table>
    <tr>
        <th>Attribute</th>
        <th>Value</th>
    </tr>
    <tr>
        <td><code>data-bft-goal</code></td>
        <td>A validation used to determine if a program is correct</td>
    </tr>
</table>

Goals
- `stack==[<values>]` - Assert that the stack must contain exactly these items where `<values>` is a comma separated list of decimal integers.

<hr>

### Cell (`bft-cell`)

A cell in the program space. A slot in which a `bft-tile` can be placed.

<table>
    <tr>
        <th>Attribute</th>
        <th>Default</th>
        <th>Value</th>
    </tr>
    <tr>
        <td><code>data-bft-goal</code></td>
        <td><code>"true"</code></td>
        <td>Tiles cannot be dragged from or to a fixed cell</td>
    </tr>
</table>

<hr>

### Tile (`bft-tile`)

A "tile"

<table>
    <tr>
        <th>Attribute</th>
        <th>Value</th>
    </tr>
    <tr>
        <td><code>data-bft-value</code></td>
        <td>The befunge instruction the tile represents</td>
    </tr>
</table>

### Inventory (`bft-inventory`)

A container for tiles not in use.

