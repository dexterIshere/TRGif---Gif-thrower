import { For, createSignal, mapArray } from "solid-js";
import "./styles/emocomp.css";
import { FaSolidPlus } from "solid-icons/fa";
import { invoke } from "@tauri-apps/api/tauri";

export const Emoc = () => {
  const [valeur, setValeur] = createSignal("");
  const [emotion, setEmotion] = createSignal("");

  async function add_gif() {
    await invoke("add_to_list", { emotion: emotion(), valeur: valeur() });
  }

  const [emo] = createSignal(["fun", "cringe", "choked-boar"]);

  const emoMap = mapArray(emo, (emotions) => {
    return (
      <div class="emoZone">
        <div class="emoContainer">
          <div class="emoInf">{emotions}</div>

          <form
            class="emoForm"
            onSubmit={(e) => {
              e.preventDefault();
              setEmotion(emotions);
              add_gif();
            }}
          >
            <input
              class="emoInput"
              id="add-gif-input"
              onChange={(e) => {
                setValeur(e.currentTarget.value);
              }}
              placeholder="Enter a gif link..."
            />
            <button class="addEmoGif" type="submit">
              ADD
            </button>
          </form>
        </div>

        <CommandKey />
      </div>
    );
  });

  const [isFormVisible, setIsFormVisible] = createSignal(false);

  return (
    <div class="emoSpwn">
      <div class="emoMap">
        <For each={emoMap()} fallback={<div> Loading... </div>}>
          {(emotions) => emotions}
        </For>
      </div>
      <button
        onClick={() => {
          setIsFormVisible(!isFormVisible());
        }}
        class="add-emo"
      >
        <FaSolidPlus />
      </button>
      {isFormVisible() && <NewEmoForm setIsFormVisible={setIsFormVisible} />}
    </div>
  );
};

export const CommandKey = () => {
  const [key, setKey] = createSignal("");
  async function newKeys() {
    setKey(await invoke("new_keys"));
  }
  return (
    <button onClick={() => newKeys()} class="KeyBTN">
      <p>{key()}</p>
    </button>
  );
};

interface NewEmoFormProps {
  setIsFormVisible: (visible: boolean) => void;
}

export function NewEmoForm({ setIsFormVisible }: NewEmoFormProps) {
  const [emoName, setEmoName] = createSignal("");

  async function newEmo() {
    await invoke("new_emo", { name: emoName() });
  }

  return (
    <div class="new-emo-form-container">
      <form
        onSubmit={(e) => {
          e.preventDefault();
          newEmo();
          setIsFormVisible(false);
        }}
      >
        <input
          onChange={(e) => {
            setEmoName(e.currentTarget.value);
          }}
          placeholder="name ..."
          type="text"
        />
        <button type="submit">new</button>
      </form>
    </div>
  );
}
