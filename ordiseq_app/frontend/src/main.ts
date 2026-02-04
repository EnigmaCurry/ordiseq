import "./app.css";
import App from "./App.svelte";
import ShaderBackground from "./lib/ShaderBackground.svelte";
import { mount } from "svelte";

// Create shader background container
const shaderContainer = document.createElement("div");
shaderContainer.id = "shader-container";
document.body.prepend(shaderContainer);

// Create overlay element
const overlay = document.createElement("div");
overlay.id = "shader-overlay";
document.body.insertBefore(overlay, document.getElementById("app"));

// Mount shader background
const shaderBackground = mount(ShaderBackground, {
  target: shaderContainer,
});

// Mount main app
const app = mount(App, {
  target: document.getElementById("app")!,
});

export { shaderBackground };
export default app;
