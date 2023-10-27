import { Emoc } from "./components/EmoC";
import { appWindow } from "@tauri-apps/api/window";
import { FaSolidMinus } from "solid-icons/fa";
import { TbMaximize } from "solid-icons/tb";
import { IoClose } from "solid-icons/io";
function App() {
  return (
    <div class="container">
      <div data-tauri-drag-region class="titlebar">
        <div
          onClick={() => appWindow.minimize()}
          class="titlebar-button"
          id="titlebar-minimize"
        >
          <FaSolidMinus />
        </div>
        <div
          onClick={() => appWindow.toggleMaximize()}
          class="titlebar-button"
          id="titlebar-maximize"
        >
          <TbMaximize />
        </div>
        <div
          onClick={() => appWindow.close()}
          class="titlebar-button"
          id="titlebar-close"
        >
          <IoClose />
        </div>
      </div>
      <Emoc />
    </div>
  );
}

export default App;
