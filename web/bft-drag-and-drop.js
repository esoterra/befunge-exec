document.addEventListener('DOMContentLoaded', onload, true);

const TILE_VALUE_TRANSFER_TYPE = "application/x-bft-tile-value";
const TILE_GROUP_TRANSFER_TYPE = "application/x-bft-tile-group";

const TILE_VALUE_ATTRIBUTE = "data-bft-value";
const CELL_FIXED_ATTRIBUTE = "data-bft-fixed";
const GROUP_ATTRIBUTE = "data-bft-group";

function onload() {
    var swapValue = undefined;

    document.querySelectorAll(".bft-tile").forEach((tile) => {
        tile.draggable = true;
    });

    document.addEventListener("dragstart", (event) => {
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

        const group = groupForElement(target);
        if (group !== null) {
            event.dataTransfer.setData(TILE_GROUP_TRANSFER_TYPE, group);
        }

        swapValue = undefined;
    });

    const dragUpdate = (event) => {
        swapValue = undefined;
        if (!event.dataTransfer.types.includes(TILE_VALUE_TRANSFER_TYPE)) {
            return;
        }
        const { target, validTarget, willSwap, targetsTile } = analyzeDropDestination(event);

        if (!validTarget) {
            return;
        }

        if (willSwap) {
            if (targetsTile) {
                swapValue = target.getAttribute(TILE_VALUE_ATTRIBUTE);
            } else {
                swapValue = target.children[0].getAttribute(TILE_VALUE_ATTRIBUTE);
            }
        }

        event.dataTransfer.dropEffect = "move";

        event.preventDefault();
    };

    document.addEventListener("dragenter", dragUpdate);
    document.addEventListener("dragover", dragUpdate);

    document.addEventListener("drop", (event) => {
        if (!event.dataTransfer.types.includes(TILE_VALUE_TRANSFER_TYPE)) {
            console.log("Couldn't drop!! No tile value");
            return;
        }
        const { target, willSwap, targetsTile } = analyzeDropDestination(event);

        const data = event.dataTransfer.getData(TILE_VALUE_TRANSFER_TYPE);

        if (targetsTile) {
            // Replace/update targetted tile
            target.setAttribute(TILE_VALUE_ATTRIBUTE, data)
            target.innerHTML = data;
        } else {
            if (willSwap) {
                // Replace/update tile in cell
                const tile = target.children[0];
                tile.setAttribute(TILE_VALUE_ATTRIBUTE, data)
                tile.innerHTML = data;
            } else {
                // Place new tile in cell
                const newTile = document.createElement("DIV");
                newTile.classList.add("bft-tile");
                newTile.draggable = true;
                newTile.setAttribute(TILE_VALUE_ATTRIBUTE, data)
                newTile.innerHTML = data;

                target.appendChild(newTile);
            }
        }
        
        event.preventDefault();
    })

    document.addEventListener("dragend", (event) => {
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

        if (swapValue !== undefined) {
            target.setAttribute(TILE_VALUE_ATTRIBUTE, swapValue);
            target.innerHTML = swapValue;
            swapValue = undefined;
        } else {
            target.remove();
        }
        
        event.preventDefault();
    })
}

const analyzeDropDestination = (event) => {
    const target = event.target;

    if (target === null || !(target instanceof Element)) {
        return; // We are only interested in targets that are elements.
    }

    let groupMatch = true;
    if (event.dataTransfer.types.includes(TILE_GROUP_TRANSFER_TYPE)) {
        const group = event.dataTransfer.getData(TILE_GROUP_TRANSFER_TYPE);
        const targetGroup = groupForElement(target);
        if (group !== targetGroup) {
            groupMatch = false;
        }
    }

    const targetsCell = target.classList.contains("bft-cell");
    const targetsTile = target.classList.contains("bft-tile");
    const targetsInventory = target.classList.contains("bft-inventory");

    const targetsMovableTile = targetsTile
        && target.parentElement.getAttribute(CELL_FIXED_ATTRIBUTE) !== "true";
    const targetsEmptyCell = targetsCell
        && target.children.length === 0;
    const targetsCellWithMovableTile = targetsCell
        && target.children.length === 1
        && target.children[0].getAttribute(CELL_FIXED_ATTRIBUTE) !== "true";

    const validTarget = groupMatch && (targetsEmptyCell || targetsCellWithMovableTile || targetsMovableTile || targetsInventory);

    const willSwap = targetsCellWithMovableTile || targetsMovableTile;

    return { target, groupMatch, validTarget, willSwap, targetsTile }
}

function groupForElement(element) {
    if (element.hasAttribute(GROUP_ATTRIBUTE)) {
        return element.getAttribute(GROUP_ATTRIBUTE);
    }

    if (element.parentElement !== null) {
        return groupForElement(element.parentElement);
    } else {
        return null;
    }
}
