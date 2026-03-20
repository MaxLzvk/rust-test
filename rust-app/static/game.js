// === Maze Runner - Linux Commands ===

const TILE = 40;
const COLS = 19;
const ROWS = 13;
const W = COLS * TILE;
const MAZE_H = ROWS * TILE;

const k = kaplay({
    width: W,
    height: MAZE_H,
    background: [13, 17, 23],
    crisp: true,
    canvas: document.getElementById("game-canvas"),
});

// Load sprite strips (4 frames each, 32x32)
loadSprite("kart_walk", "sprites/kart_walk.png", {
    sliceX: 4, anims: { walk: { from: 0, to: 3, loop: true, speed: 6 } }
});
loadSprite("megaman_walk", "sprites/megaman_walk.png", {
    sliceX: 4, anims: { walk: { from: 0, to: 3, loop: true, speed: 8 } }
});
loadSprite("slime_walk", "sprites/slime_walk.png", {
    sliceX: 4, anims: { walk: { from: 0, to: 3, loop: true, speed: 6 } }
});
loadSprite("mage_walk", "sprites/mage_walk.png", {
    sliceX: 4, anims: { walk: { from: 0, to: 3, loop: true, speed: 8 } }
});
loadSprite("mermaid_walk", "sprites/mermaid_walk.png", {
    sliceX: 4, anims: { walk: { from: 0, to: 3, loop: true, speed: 4 } }
});

const CHARACTERS = [
    { name: "Kart",       color: [231, 76, 60],   sprite: "kart_walk" },
    { name: "Mega Man",   color: [52, 152, 219],  sprite: "megaman_walk" },
    { name: "Slime",      color: [46, 204, 113],  sprite: "slime_walk" },
    { name: "Mage",       color: [155, 89, 182],  sprite: "mage_walk" },
    { name: "Sirene",     color: [52, 180, 180],  sprite: "mermaid_walk" },
];

const LICENSES = [
    "Voiture: Mario Kart (Nintendo) - spriters-resource.com",
    "Mega Man: Capcom - spriters-resource.com",
    "Slime: Dragon Quest (Square Enix) - spriters-resource.com",
    "Mage: Final Fantasy (Square Enix) - spriters-resource.com",
    "Sirene: Puzzle & Dragons (GungHo) - spriters-resource.com",
    "",
    "Sprites utilises a titre educatif uniquement.",
    "Maze Runner - CFPT Geneve - Cours Infra 2eme annee",
];

// === HTML Inventory System ===
const invContainer = document.getElementById("inv-items");
const scoreDisplay = document.getElementById("score-display");
const flashOverlay = document.getElementById("flash-overlay");

let inventory = [];
let selected = new Set();
let score = 0;
let matching = false;

function flashScreen(type) {
    flashOverlay.className = type;
    setTimeout(() => { flashOverlay.className = ""; }, 300);
}

function renderInventory() {
    invContainer.innerHTML = "";
    inventory.forEach((item, i) => {
        const div = document.createElement("div");
        div.className = "inv-item " + item.kind.substring(0, 3) + (selected.has(i) ? " selected" : "");

        const tag = document.createElement("span");
        tag.className = "type-tag";
        tag.textContent = item.kind === "command" ? "COMMANDE" : "DEFINITION";
        div.appendChild(tag);

        const label = document.createElement("span");
        label.textContent = item.label;
        div.appendChild(label);

        div.addEventListener("click", () => onInvClick(i));
        invContainer.appendChild(div);
    });
}

function onInvClick(idx) {
    if (matching) return;

    if (selected.has(idx)) {
        selected.delete(idx);
        renderInventory();
        return;
    }

    selected.add(idx);

    if (selected.size < 2) {
        renderInventory();
        return;
    }

    const indices = Array.from(selected);
    const a = inventory[indices[0]];
    const b = inventory[indices[1]];

    let cmd, def;
    if (a.kind === "command" && b.kind === "definition") {
        cmd = a.label; def = b.label;
    } else if (b.kind === "command" && a.kind === "definition") {
        cmd = b.label; def = a.label;
    } else {
        flashScreen("red");
        selected.clear();
        renderInventory();
        return;
    }

    matching = true;
    fetch("api/validate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ command: cmd, definition: def }),
    })
    .then(r => r.json())
    .then(result => {
        if (result.correct) {
            score++;
            scoreDisplay.textContent = "Score: " + score + " / 6";
            flashScreen("green");
            const [hi, lo] = indices[0] > indices[1] ? [indices[0], indices[1]] : [indices[1], indices[0]];
            inventory.splice(hi, 1);
            inventory.splice(lo, 1);
            if (score >= 6) {
                setTimeout(() => go("win", { score }), 800);
            }
        } else {
            flashScreen("red");
        }
        selected.clear();
        matching = false;
        renderInventory();
    })
    .catch(() => {
        flashScreen("red");
        selected.clear();
        matching = false;
        renderInventory();
    });
}

// === Maze generation ===
function generateMaze(cols, rows) {
    const w = Math.floor(cols / 2);
    const h = Math.floor(rows / 2);
    const cells = Array.from({ length: h }, () => Array(w).fill(false));
    const walls = {
        h: Array.from({ length: h + 1 }, () => Array(w).fill(true)),
        v: Array.from({ length: h }, () => Array(w + 1).fill(true)),
    };
    const dirs = [[0, -1], [0, 1], [-1, 0], [1, 0]];

    function shuffle(arr) {
        for (let i = arr.length - 1; i > 0; i--) {
            const j = Math.floor(Math.random() * (i + 1));
            [arr[i], arr[j]] = [arr[j], arr[i]];
        }
        return arr;
    }

    const stack = [];
    function carve(x, y) {
        cells[y][x] = true;
        stack.push([x, y]);
        while (stack.length > 0) {
            const [cx, cy] = stack[stack.length - 1];
            const nb = shuffle([...dirs])
                .map(([dx, dy]) => [cx + dx, cy + dy, dx, dy])
                .filter(([nx, ny]) => nx >= 0 && nx < w && ny >= 0 && ny < h && !cells[ny][nx]);
            if (nb.length === 0) { stack.pop(); continue; }
            const [nx, ny, dx, dy] = nb[0];
            if (dx === 0 && dy === -1) walls.h[cy][cx] = false;
            if (dx === 0 && dy === 1) walls.h[cy + 1][cx] = false;
            if (dx === -1 && dy === 0) walls.v[cy][cx] = false;
            if (dx === 1 && dy === 0) walls.v[cy][cx + 1] = false;
            cells[ny][nx] = true;
            stack.push([nx, ny]);
        }
    }
    carve(0, 0);

    const grid = Array.from({ length: rows }, () => Array(cols).fill(1));
    for (let y = 0; y < h; y++) {
        for (let x = 0; x < w; x++) {
            grid[y * 2 + 1][x * 2 + 1] = 0;
            if (!walls.h[y][x]) grid[y * 2][x * 2 + 1] = 0;
            if (!walls.h[y + 1][x]) grid[y * 2 + 2][x * 2 + 1] = 0;
            if (!walls.v[y][x]) grid[y * 2 + 1][x * 2] = 0;
            if (!walls.v[y][x + 1]) grid[y * 2 + 1][x * 2 + 2] = 0;
        }
    }
    return grid;
}

function getFreeCells(grid) {
    const free = [];
    for (let y = 0; y < grid.length; y++)
        for (let x = 0; x < grid[y].length; x++)
            if (grid[y][x] === 0 && !(x === 1 && y === 1)) free.push({ x, y });
    for (let i = free.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [free[i], free[j]] = [free[j], free[i]];
    }
    return free;
}

// === SCENES ===

scene("start", () => {
    // Reset HTML state
    invContainer.innerHTML = "";
    inventory = [];
    selected.clear();
    score = 0;
    matching = false;
    scoreDisplay.textContent = "Score: 0 / 6";

    add([text("MAZE RUNNER", { size: 42 }), pos(W / 2, 60), anchor("center"), color(233, 69, 96)]);
    add([text("Trouve les commandes Linux\net leurs definitions !", { size: 22, align: "center" }), pos(W / 2, 120), anchor("center"), color(200, 200, 200)]);
    add([text("Choisis ton personnage", { size: 20 }), pos(W / 2, 180), anchor("center"), color(150, 150, 150)]);

    CHARACTERS.forEach((char, i) => {
        const x = W / 2 + (i - 2) * 120;
        const y = 280;
        add([rect(68, 68, { radius: 8 }), pos(x, y), anchor("center"), color(30, 36, 45), area(), "charBtn", { charIndex: i }]);
        add([sprite(char.sprite, { width: 48, height: 48, anim: "walk" }), pos(x, y), anchor("center")]);
        add([text(char.name, { size: 14 }), pos(x, y + 50), anchor("center"), color(200, 200, 200)]);
    });
    onClick("charBtn", (btn) => go("game", { charIndex: btn.charIndex }));

    add([rect(90, 34, { radius: 4 }), pos(W - 55, MAZE_H - 35), anchor("center"), color(40, 40, 50), area(), "aboutBtn"]);
    add([text("About", { size: 18 }), pos(W - 55, MAZE_H - 35), anchor("center"), color(150, 150, 150)]);
    onClick("aboutBtn", () => go("about"));
});

scene("about", () => {
    add([rect(W, MAZE_H), pos(0, 0), color(13, 17, 23)]);
    add([text("Credits & Licences", { size: 28 }), pos(W / 2, 30), anchor("center"), color(233, 69, 96)]);
    LICENSES.forEach((line, i) => {
        add([text(line, { size: 16 }), pos(30, 80 + i * 32), color(200, 200, 200)]);
    });
    add([rect(130, 40, { radius: 4 }), pos(W / 2, MAZE_H - 50), anchor("center"), color(40, 40, 50), area(), "backBtn"]);
    add([text("Retour", { size: 20 }), pos(W / 2, MAZE_H - 50), anchor("center"), color(200, 200, 200)]);
    onClick("backBtn", () => go("start"));
});

scene("game", async ({ charIndex }) => {
    const char = CHARACTERS[charIndex];
    const grid = generateMaze(COLS, ROWS);

    // Reset HTML
    inventory = [];
    selected.clear();
    matching = false;
    renderInventory();

    // Draw maze
    for (let y = 0; y < ROWS; y++) {
        for (let x = 0; x < COLS; x++) {
            add([
                rect(TILE, TILE),
                pos(x * TILE, y * TILE),
                color(grid[y][x] === 1 ? 30 : 22, grid[y][x] === 1 ? 36 : 27, grid[y][x] === 1 ? 45 : 34),
            ]);
        }
    }

    // Player
    let playerX = 1, playerY = 1, moveCD = 0;
    const playerObj = add([
        sprite(char.sprite, { width: TILE - 4, height: TILE - 4, anim: "walk" }),
        pos(playerX * TILE + 2, playerY * TILE + 2),
        z(60),
    ]);

    function tryMove(dx, dy) {
        const nx = playerX + dx, ny = playerY + dy;
        if (nx >= 0 && nx < COLS && ny >= 0 && ny < ROWS && grid[ny][nx] === 0) {
            playerX = nx; playerY = ny;
            playerObj.pos.x = nx * TILE + 2;
            playerObj.pos.y = ny * TILE + 2;
            checkItemPickup();
        }
    }

    onUpdate(() => {
        if (moveCD > 0) { moveCD -= dt(); return; }
        const delay = 0.12;
        if (isKeyDown("left") || isKeyDown("a")) { tryMove(-1, 0); moveCD = delay; }
        else if (isKeyDown("right") || isKeyDown("d")) { tryMove(1, 0); moveCD = delay; }
        else if (isKeyDown("up") || isKeyDown("w")) { tryMove(0, -1); moveCD = delay; }
        else if (isKeyDown("down") || isKeyDown("s")) { tryMove(0, 1); moveCD = delay; }
    });

    // Fetch maze items
    let mazeData = [];
    try {
        const resp = await fetch("api/maze");
        mazeData = await resp.json();
    } catch (e) { mazeData = []; }

    const freeCells = getFreeCells(grid);
    const itemObjects = new Map();

    mazeData.forEach((item, i) => {
        if (i >= freeCells.length) return;
        const cell = freeCells[i];
        const isCmd = item.kind === "command";

        const obj = add([
            rect(TILE - 4, TILE - 4, { radius: 4 }),
            pos(cell.x * TILE + 2, cell.y * TILE + 2),
            color(isCmd ? 46 : 100, isCmd ? 160 : 60, isCmd ? 90 : 140),
            z(50),
            "mazeItem",
        ]);

        add([
            text(isCmd ? item.label : "?", { size: 13 }),
            pos(cell.x * TILE + TILE / 2, cell.y * TILE + TILE / 2),
            anchor("center"), color(255, 255, 255), z(51),
            "itemLabel_" + i,
        ]);

        itemObjects.set(i, { obj, cell, item });
    });

    function checkItemPickup() {
        for (const [idx, data] of itemObjects) {
            if (data.cell.x === playerX && data.cell.y === playerY) {
                inventory.push({ label: data.item.label, kind: data.item.kind, pair_id: data.item.pair_id });
                destroyAll("itemLabel_" + idx);
                destroy(data.obj);
                itemObjects.delete(idx);
                renderInventory();
                break;
            }
        }
    }

    // About button
    add([rect(70, 26, { radius: 4 }), pos(8, 8), color(40, 40, 50), area(), z(100), "aboutBtnGame"]);
    add([text("About", { size: 14 }), pos(43, 21), anchor("center"), color(150, 150, 150), z(100)]);
    onClick("aboutBtnGame", () => go("about"));
});

scene("win", ({ score }) => {
    invContainer.innerHTML = '<div style="text-align:center;padding:20px;color:#2ecc71;font-size:24px;">Bravo ! Toutes les paires trouvees !</div>';

    add([text("Bravo !", { size: 52 }), pos(W / 2, MAZE_H / 3), anchor("center"), color(46, 204, 113)]);
    add([text("Tu as trouve les " + score + " paires !", { size: 24 }), pos(W / 2, MAZE_H / 3 + 70), anchor("center"), color(200, 200, 200)]);
    add([rect(170, 44, { radius: 6 }), pos(W / 2, MAZE_H / 2 + 50), anchor("center"), color(233, 69, 96), area(), "replayBtn"]);
    add([text("Rejouer", { size: 22 }), pos(W / 2, MAZE_H / 2 + 50), anchor("center"), color(255, 255, 255)]);
    onClick("replayBtn", () => {
        fetch("api/score/reset", { method: "POST" });
        go("start");
    });
});

go("start");
