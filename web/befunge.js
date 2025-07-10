document.addEventListener('DOMContentLoaded', onload, true);

const TILE_VALUE_TRANSFER_TYPE = "application/x-bft-tile-value";

const TILE_VALUE_ATTRIBUTE = "data-bft-value";
const CELL_FIXED_ATTRIBUTE = "data-bft-fixed";

function onload() {
    document.querySelectorAll(".bft-tile").forEach((tile) => {
        console.log(tile);
        tile.draggable = true;
    });

    document.addEventListener("dragstart", (event) => {
        console.log(event);
        const target = event.target;
        if (target === null || !(target instanceof Element)) {
            console.log("Target null or not an Element");
            return; // We are only interested in targets that are elements.
        }
        if (!target.classList.contains("bft-tile")) {
            console.log("Target is not a tile");
            return; // We are only interested in .bft-tile elements.
        }
        const value = target.getAttribute(TILE_VALUE_ATTRIBUTE);
        if (value === null) {
            console.log("No bft-value found");
            return; // Ignore tiles without `data-bft-value` set.
        }

        const parentFixed = target.parentElement.getAttribute(CELL_FIXED_ATTRIBUTE);
        if (parentFixed === "true") {
            console.log("Can't move fixed.");
            return;
        }

        event.dataTransfer.setData(TILE_VALUE_TRANSFER_TYPE, value);
        event.dataTransfer.effectAllowed = "move";
    });

    const dragUpdate = (event) => {
        console.log(event);
        if (!event.dataTransfer.types.includes(TILE_VALUE_TRANSFER_TYPE)) {
            return;
        }
        const target = event.target;
        if (target === null || !(target instanceof Element)) {
            return; // We are only interested in targets that are elements.
        }
        const legalCell = target.classList.contains("bft-cell") && target.children.length == 0;
        const legalInventory = target.classList.contains("bft-inventory");

        if (legalCell || legalInventory) {
            event.preventDefault();
        }
    };

    document.addEventListener("dragenter", dragUpdate);
    document.addEventListener("dragover", dragUpdate);

    document.addEventListener("drop", (event) => {
        console.log(event);
        if (!event.dataTransfer.types.includes(TILE_VALUE_TRANSFER_TYPE)) {
            console.log("Couldn't drop!! No tile value");
            return;
        }
        const target = event.target;
        if (target === null || !(target instanceof Element)) {
            console.log("Couldn't drop!! Target is not element");
            return; // We are only interested in targets that are elements.
        }
        const data = event.dataTransfer.getData(TILE_VALUE_TRANSFER_TYPE);

        // Make destination tile
        const newTile = document.createElement("DIV");
        newTile.classList.add("bft-tile");
        newTile.draggable = true;
        newTile.setAttribute(TILE_VALUE_ATTRIBUTE, data)
        newTile.innerHTML = data;

        target.appendChild(newTile);
        event.preventDefault();
    })

    document.addEventListener("dragend", (event) => {
        console.log(event);
        if (!event.dataTransfer.types.includes(TILE_VALUE_TRANSFER_TYPE)) {
            return;
        }
        if (event.dataTransfer.dropEffect === "none") {
            return;
        }
        const target = event.target;
        if (target === null || !(target instanceof Element)) {
            return; // We are only interested in targets that are elements.
        }
        target.remove();
        event.preventDefault();
    })
}
