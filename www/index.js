// @ts-check
import * as wasm from "fontdue-you-see-it";
import {memory} from "fontdue-you-see-it/fontdue_you_see_it_bg";

function getCanvasA() {
    const existing = document.getElementById("canvas-a");
    if (existing) {
        return /** @type {HTMLCanvasElement} */(existing);
    }
    const elem = document.createElement("canvas");
    elem.id = "canvas-a";
    document.getElementById("samples").appendChild(elem);
    return elem;
}

function getCanvasB() {
    const existing = document.getElementById("canvas-b");
    if (existing) {
        return /** @type {HTMLCanvasElement} */(existing);
    }
    const elem = document.createElement("canvas");
    elem.id = "canvas-b";
    document.getElementById("samples").appendChild(elem);
    return elem;
}

/**
 * @param {Uint8ClampedArray} buffer 
 * @param {number} width 
 * @param {number} height 
 */
function logFormatted(buffer, width, height) {
    let lines = [];
    for (let y = 0; y < height; y++) {
        let line = [];
        for (let x = 0; x < width; x++) {
            let val = "" + buffer[y * width + x];
            while (val.length < 3) {
                val = " " + val;
            }
            line.push(val);
        }
        lines.push(line.join(" "));
    }
    console.log(lines.join("\n"))
    console.log(width, height);
}

function renderFontdueCharacter(char = "¾", size = 600) {
    const rednerResult = wasm.render(size, char);
    const textureRaw = new Uint8ClampedArray(memory.buffer, rednerResult.bitmap.offset(), rednerResult.bitmap.size());
    const clampedFullColor = new Uint8ClampedArray(textureRaw.length * 4);
    for (let i = 0; i < textureRaw.length; i++) {
        clampedFullColor[i*4] = 255 - textureRaw[i];
        clampedFullColor[i*4 + 1] = 255 - textureRaw[i];
        clampedFullColor[i*4 + 2] = 255 - textureRaw[i];
        clampedFullColor[i*4 + 3] = 255;
    }
    const image = new ImageData(clampedFullColor, rednerResult.width, rednerResult.height);
    const elem = getCanvasA();
    elem.height = rednerResult.height + 20;
    elem.width = rednerResult.width + 20;
    const ctx = elem.getContext("2d");
    ctx.putImageData(image, 10, 10);
    const [xmin, ymin] = [rednerResult.xmin, rednerResult.ymin];
    rednerResult.free();
    return [elem.height, elem.width, xmin, ymin];
}

function renderBuiltinCharacter(char = "¾", xmin, ymin, height = 200, width = 200, size = 600) {
    const elem = getCanvasB();
    elem.height = height;
    elem.width = width;
    const ctx = elem.getContext("2d");
    ctx.font = `${size}px 'Roboto Mono'`;
    ctx.fillText(char, 10 - xmin, height + ymin - 10);
}

const input = /** @type {HTMLInputElement} */(document.getElementById("input"));
input.addEventListener("change", function () {
    const [height, width, xmin, ymin] = renderFontdueCharacter(this.value);
    renderBuiltinCharacter(this.value, xmin, ymin, height, width);
});
const [height, width, xmin, ymin] = renderFontdueCharacter();
renderBuiltinCharacter(undefined, xmin, ymin, height, width);
