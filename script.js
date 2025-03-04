import init, { get_paths } from "./wasm-paths/pkg/wasm_paths.js";

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");
let wasmInitialized = false;

let rect = {
  x: 100,
  y: 100,
  width: 600,
  height: 400,
};
const edgeThreshold = 8;

let isResizing = false;
let resizeEdge = "";
let lastMouseX = 0;
let lastMouseY = 0;

async function initWasm() {
  try {
    await init();
    wasmInitialized = true;
    draw();
  } catch (e) {
    console.error("Error initializing WASM:", e);
  }
}

function resizeCanvas() {
  const container = document.getElementById("canvas-container");
  canvas.width = container.clientWidth;
  canvas.height = container.clientHeight;
  draw();
}

function draw() {
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  ctx.strokeStyle = "#0078d7";
  ctx.lineWidth = 2;
  ctx.strokeRect(rect.x, rect.y, rect.width, rect.height);

  if (wasmInitialized) {
    drawPaths();
  }
}

function inputForInputType(inputType) {
  switch (inputType) {
    case "option1":
      return 0;
    case "option2":
      return 1;
    case "option3":
      return 2;
    case "option4":
      return 3;
    default:
      return 0;
  }
}

function drawPaths() {
  const input = inputForInputType(document.getElementById("inputType").value);
  const textSize = parseInt(document.getElementById("textSize").value);
  const textColor = document.getElementById("textColor").value;
  const paths = get_paths(
    rect.x,
    rect.y,
    rect.width,
    rect.height,
    textSize,
    input
  );

  document.getElementById("textSizeStr").innerText = `${textSize}`;

  ctx.strokeStyle = "none";
  ctx.fillStyle = textColor;

  for (let path of paths) {
    const p = new Path2D(path);
    ctx.fill(p);
  }
}

function getResizeEdge(x, y) {
  let edge = "";

  if (
    Math.abs(x - rect.x) <= edgeThreshold &&
    y >= rect.y &&
    y <= rect.y + rect.height
  ) {
    edge += "left";
  } else if (
    Math.abs(x - (rect.x + rect.width)) <= edgeThreshold &&
    y >= rect.y &&
    y <= rect.y + rect.height
  ) {
    edge += "right";
  }

  if (
    Math.abs(y - rect.y) <= edgeThreshold &&
    x >= rect.x &&
    x <= rect.x + rect.width
  ) {
    edge += "top";
  } else if (
    Math.abs(y - (rect.y + rect.height)) <= edgeThreshold &&
    x >= rect.x &&
    x <= rect.x + rect.width
  ) {
    edge += "bottom";
  }

  return edge;
}

function updateCursor(edge, mouseX, mouseY) {
  if (edge === "left" || edge === "right") {
    canvas.style.cursor = "ew-resize";
  } else if (edge === "top" || edge === "bottom") {
    canvas.style.cursor = "ns-resize";
  } else if (edge === "topleft" || edge === "bottomright") {
    canvas.style.cursor = "nwse-resize";
  } else if (edge === "topright" || edge === "bottomleft") {
    canvas.style.cursor = "nesw-resize";
  } else if (
    mouseX > rect.x + edgeThreshold &&
    mouseX < rect.x + rect.width - edgeThreshold &&
    mouseY > rect.y + edgeThreshold &&
    mouseY < rect.y + rect.height - edgeThreshold
  ) {
    canvas.style.cursor = "text";
  } else {
    canvas.style.cursor = "default";
  }
}

canvas.addEventListener("mousedown", (e) => {
  const rect = canvas.getBoundingClientRect();
  const mouseX = e.clientX - rect.left;
  const mouseY = e.clientY - rect.top;

  resizeEdge = getResizeEdge(mouseX, mouseY);

  if (resizeEdge) {
    isResizing = true;
    lastMouseX = mouseX;
    lastMouseY = mouseY;
  }
});

canvas.addEventListener("mousemove", (e) => {
  const canvasRect = canvas.getBoundingClientRect();
  const mouseX = e.clientX - canvasRect.left;
  const mouseY = e.clientY - canvasRect.top;

  if (isResizing) {
    if (resizeEdge.includes("left")) {
      const widthChange = lastMouseX - mouseX;
      rect.width += widthChange;
      rect.x -= widthChange;
    } else if (resizeEdge.includes("right")) {
      rect.width = mouseX - rect.x;
    }

    if (resizeEdge.includes("top")) {
      const heightChange = lastMouseY - mouseY;
      rect.height += heightChange;
      rect.y -= heightChange;
    } else if (resizeEdge.includes("bottom")) {
      rect.height = mouseY - rect.y;
    }

    if (rect.width < 20) rect.width = 20;
    if (rect.height < 20) rect.height = 20;

    lastMouseX = mouseX;
    lastMouseY = mouseY;
    draw();
  } else {
    const edge = getResizeEdge(mouseX, mouseY);
    updateCursor(edge, mouseX, mouseY);
  }
});

canvas.addEventListener("mouseup", () => {
  isResizing = false;
});

canvas.addEventListener("mouseleave", () => {
  isResizing = false;
  canvas.style.cursor = "default";
});

document.getElementById("inputType").addEventListener("change", draw);
document.getElementById("textSize").addEventListener("change", draw);
document.getElementById("textColor").addEventListener("change", draw);

window.addEventListener("load", () => {
  resizeCanvas();
  window.addEventListener("resize", resizeCanvas);
  initWasm();
});
