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

        const { value, validSource } = analyzeDropSource(event);
        if (!validSource) {
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
        // console.log(event);
        swapValue = undefined;
        if (!event.dataTransfer.types.includes(TILE_VALUE_TRANSFER_TYPE)) {
            return;
        }
        const { target, validTarget, willSwap, targetsTile } = analyzeDropDestination(event);

        if (!validTarget) {
            console.log("target not valid")
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
        // console.log(event); 
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
        // console.log(event);
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

const analyzeDropSource = (event) => {
    const target = event.target;

    if (target === null || !(target instanceof Element)) {
        // We are only interested in targets that are elements.
        console.log("Target null or not an Element");
        return { validSource: false }; 
    }
    if (!target.classList.contains("bft-tile")) {
        // We are only interested in .bft-tile elements.
        console.log("Target is not a tile");
        return { validSource: false }; 
    }
    const value = target.getAttribute(TILE_VALUE_ATTRIBUTE);
    if (value === null) {
        // Ignore tiles without `data-bft-value` set.
        console.log("No bft-value found");
        return { validSource: false }; 
    }

    const parentFixed = target.parentElement.getAttribute(CELL_FIXED_ATTRIBUTE);
    if (parentFixed === "true") {
        console.log("Can't move fixed.");
        return { validSource: false }; 
    }

    return { value, validSource: true }; 
};

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

function rootsForElement(element) {
    const group = groupForElement(element);
    return document.querySelectorAll(`[data-bft-group="${group}"]`)
}

const successClass = 'bft-success';
const failClass = 'bft-fail'

function updateRoots(element, success, fail) {
    rootsForElement(element).forEach((element) => {
        if (success) {
            element.classList.add(successClass);
        } else {
            element.classList.remove(successClass);
        }
        if (fail) {
            element.classList.add(failClass);
        } else {
            element.classList.remove(failClass)
        }
    });
}

class Clock {
    id;

    constructor(func, delay) {
        this.id = setInterval(func, delay);
    }

    stop() {
        if (this.id) {
            clearInterval(this.id);
            this.id = false;
        }
    }
}

class Interpreter {
    group;
    state;
    clock;

    x;
    y;

    constructor(group) {
        this.group = group;
        this.state = 'stopped';
        this.clock = undefined;

        this.x = 0;
        this.y = 0;
    }

    stop() {

    }

    start() {

    }

    reset() {
        // Empty the stack
        this.stack.replaceChildren()

        // Move cursor to beginning
        this.getCell(this.x, this.y).classList.remove('bft-cursor');
        this.getCell(0, 0).classList.add('bft-cursor');
        this.x = 0;
        this.y = 0;
        
        // Update the state to stopped
        this.getAllRoots().forEach((root) => {
            root.classList.remove('bft-running');
            root.classList.add('bft-stopped');
        })
    }

    getAllRoots() {
        return document.querySelectorAll(`[data-bft-group="${group}"]`);
    }

    getButton() {
        return document.querySelector(`[data-bft-group="${group}"] .bft-button`)
    }

    getGrid() {
        return document.querySelector(`[data-bft-group="${group}"] .bft-grid`);
    }

    getStack() {
        return document.querySelector(`[data-bft-group="${group}"] .bft-stack`);
    }

    getAllCells() {
        return document.querySelectorAll(`[data-bft-group="${group}"] .bft-cell`);
    }

    getCell(x, y) {
        const grid = this.getGrid();
        const row = grid.children.item(y);
        return row.children.item(x);
    }
}

const state = new Map();

function getOrInitState(group) {
    const groupState = state.get(group);
    if (groupState) {
        return groupState;
    } else {
        const newState = new GroupState(group);
        groupState.set(group, newState);
        return newState;
    }
}

function runButtonClick(element) {
    const group = groupForElement(element);
    const groupState = getOrInitState(group);
    
    if (groupState.state === 'stopped') {
        
    }
}