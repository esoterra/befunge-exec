# Notes

# BFT Classes

### Grid (`bft-grid`)

The program space

<table>
    <tr>
        <th>Attribute</th>
        <th>Value</th>
    </tr>
    <tr>
        <td><code>data-bft-group</code></td>
        <td>Which group this grid is a part of.<br>
        Tiles must stay in their group.</td>
    </tr>
    <tr>
        <td><code>data-bft-expected-stack</code></td>
        <td>A comma separated list of the values expected to be on the stack <br>at the end of the program</td>
    </tr>
    <tr>
        <td><code>data-bft-expected-output</code></td>
        <td>The output that the program is expected to produce</td>
    </tr>
</table>

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
        <td>Tiles cannot be dragged from or<br>to a fixed cell</td>
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

<table>
    <tr>
        <th>Attribute</th>
        <th>Value</th>
    </tr>
    <tr>
        <td><code>data-bft-group</code></td>
        <td>Which group this grid is a part of.<br> Tiles must stay in their group.</td>
    </tr>
</table>

# TODO: Stacks of tiles

```html
<div class="bft-inventory">
    <div class="bft-stack" data-bft-stack-amount="10">
        <div class="bft-tile" data-bft-value="*"></div>
    </div>
</div>
```
