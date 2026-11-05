"use strict";

// Local, offline interaction demonstration only. No API or network calls.
const channels = Object.freeze([
  { name: "TRT 1", description: "Live-Fernsehen · Beispielsender" },
  { name: "ATV", description: "Live-Fernsehen · Beispielsender" },
  { name: "Kanal D", description: "Live-Fernsehen · Beispielsender" },
  { name: "Show TV", description: "Live-Fernsehen · Beispielsender" },
  { name: "Dede Film", description: "Virtueller Sender · Beispiel" },
  { name: "Teşkilat TV", description: "Virtueller Sender · Beispiel" }
]);
const $ = (id) => document.getElementById(id);
const panel = $("channel-overlay");
const items = $("channel-items");

function restoreIndex() {
  try {
    const value = Number.parseInt(localStorage.getItem("dede-demo-channel") || "2", 10);
    return Number.isInteger(value) && value >= 0 && value < channels.length ? value : 2;
  } catch {
    return 2;
  }
}

let current = restoreIndex();
let highlighted = current;
let open = false;

function pad(index) {
  return String(index + 1).padStart(2, "0");
}

function draw() {
  const channel = channels[current];
  $("now-title").textContent = channel.name;
  $("now-description").textContent = channel.description;
  $("now-number").textContent = pad(current);
  $("now-channel").textContent = channel.name;
  $("remote-title").textContent = channel.name;
  $("remote-index").textContent = `${pad(current)} / ${pad(channels.length - 1)}`;
  panel.hidden = !open;
  $("screen-help").textContent = open ? "Zurück zum Bild" : "OK für Senderliste";
  items.replaceChildren();
  if (!open) return;

  channels.forEach((item, index) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = index === highlighted ? "channel-item active" : "channel-item";
    button.setAttribute("aria-current", index === current ? "true" : "false");
    button.setAttribute("aria-label", `${pad(index)} ${item.name}`);
    const number = document.createElement("span");
    number.className = "index";
    number.textContent = pad(index);
    const title = document.createElement("span");
    title.textContent = item.name;
    button.append(number, title);
    button.addEventListener("click", () => {
      highlighted = index;
      tune(index);
    });
    items.append(button);
  });
}

function tune(index) {
  current = (index + channels.length) % channels.length;
  highlighted = current;
  open = false;
  try {
    localStorage.setItem("dede-demo-channel", String(current));
  } catch {
    // Browsers with storage disabled still support the offline preview.
  }
  draw();
}

function action(name) {
  switch (name) {
    case "open":
      highlighted = current;
      open = true;
      break;
    case "ok":
      if (open) return tune(highlighted);
      highlighted = current;
      open = true;
      break;
    case "back":
      open = false;
      break;
    case "up":
      if (!open) {
        highlighted = current;
        open = true;
      } else {
        highlighted = (highlighted - 1 + channels.length) % channels.length;
      }
      break;
    case "down":
      if (!open) {
        highlighted = current;
        open = true;
      } else {
        highlighted = (highlighted + 1) % channels.length;
      }
      break;
    case "previous":
      return tune(current - 1);
    case "next":
      return tune(current + 1);
    default:
      return;
  }
  draw();
}

document.querySelectorAll("[data-action]").forEach((button) => {
  button.addEventListener("click", () => action(button.dataset.action));
});

document.addEventListener("keydown", (event) => {
  const keyboard = {
    Enter: "ok",
    Escape: "back",
    Backspace: "back",
    ArrowUp: "up",
    ArrowDown: "down",
    PageUp: "previous",
    PageDown: "next"
  };
  const nextAction = keyboard[event.key];
  if (nextAction && !event.altKey && !event.ctrlKey && !event.metaKey) {
    event.preventDefault();
    action(nextAction);
  }
});

draw();
