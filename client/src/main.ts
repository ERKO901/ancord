import { mount } from "svelte";
import "./styles/tokens.css";
import "./styles/base.css";
import App from "./App.svelte";
import { installDevBackend } from "./lib/ipc";

await installDevBackend();
mount(App, { target: document.getElementById("app") as HTMLElement });
