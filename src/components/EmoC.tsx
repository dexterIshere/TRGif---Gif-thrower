import { For, createEffect, createSignal, mapArray } from "solid-js";
import "./styles/emocomp.css";
import { FaSolidPlus } from "solid-icons/fa";
import { invoke } from "@tauri-apps/api/tauri";
import { BsTrashFill } from "solid-icons/bs";
import { IoClose } from "solid-icons/io";
import { FiCornerRightDown } from "solid-icons/fi";
const [emoArray, setEmoArray] = createSignal<string[]>([]);

async function fetch_emo_list() {
  const result = await invoke<string>("watch_emo_folder");
  const parsedResult: string[] = JSON.parse(result);
  setEmoArray(parsedResult);
}

export const Emoc = () => {
  const [valeur, setValeur] = createSignal("");
  const [emotion, setEmotion] = createSignal("");

  async function add_gif() {
    await invoke("add_to_list", { emotion: emotion(), valeur: valeur() });
  }

  createEffect(async () => {
    fetch_emo_list();
  });

  async function rmvEmo() {
    await invoke("rmv_emo", { emotion: emotion() });
  }

  const emoMap = mapArray(emoArray, (emotions) => {
    return (
      <div class="emoZone">
        <div class="emoInf">
          <p>{emotions}</p> <FiCornerRightDown />
        </div>
        <div class="emoFlex">
          <div class="emoContainer">
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
                <FaSolidPlus />
              </button>
            </form>
          </div>
          <button
            onClick={() => {
              setEmotion(emotions);
              rmvEmo();
              fetch_emo_list();
            }}
            class="rmvEmo"
          >
            <BsTrashFill />
          </button>
          <CommandKey emotionN={emotions} />
        </div>
      </div>
    );
  });

  const [isFormVisible, setIsFormVisible] = createSignal(false);

  return (
    <div class="emoSpwn">
      <div class="emoMap">
        <For each={emoMap()} fallback={<div> No shortcuts ?? </div>}>
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

interface CommandKeyProps {
  emotionN: string;
}

export const CommandKey = ({ emotionN }: CommandKeyProps) => {
  const [key, setKey] = createSignal("");

  createEffect(async () => {
    const fetchedKey = (await invoke("fetch_emo_key", {
      emotion: emotionN,
    })) as string;
    setKey(fetchedKey);
  });

  async function newKeys() {
    setKey(await invoke("new_keys", { emotion: emotionN }));
  }

  function clearKey() {
    setKey("");
  }

  return (
    <button
      onClick={() => {
        newKeys();
        clearKey();
      }}
      class="KeyBTN"
    >
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
      <button
        class="close_creator"
        onClick={() => {
          setIsFormVisible(false);
        }}
      >
        <IoClose />
      </button>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          newEmo();
          setIsFormVisible(false);
          fetch_emo_list();
        }}
        class="new-emo-form"
      >
        <input
          onChange={(e) => {
            setEmoName(e.currentTarget.value);
          }}
          placeholder="name ..."
          type="text"
          class="new-emo-form-input"
        />
        <button type="submit" class="new-emo-form-btn">
          new
        </button>
      </form>
    </div>
  );
}
