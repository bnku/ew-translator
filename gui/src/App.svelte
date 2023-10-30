<script lang="ts">
  import { LogicalSize, appWindow } from "@tauri-apps/api/window";
  import { listen, type EventName } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  const setSize = async () => {
    const block: HTMLElement | null = document.querySelector(".translation");
    if (block) await appWindow.setSize(new LogicalSize(block.offsetWidth, block.offsetHeight));
    await appWindow.setFocus();
  };

  let translation = "";

  $: {
    translation;
    setSize();
    appWindow.setFocus();
  }

  interface translateEvent {
    event: EventName;
    payload: {
      message: string;
    };
  }

  onMount(async () => {
    await listen("translate", (event: translateEvent) => {
      console.log(event.event, event.payload, event.payload.message);
      translation = event.payload.message.replaceAll("\n", "<br/>");
      setTimeout(() => {
        setSize();
      }, 100);
    });

    appWindow.listen("tauri://focus", ({ event, payload }) => {
      console.log("focus");
      setSize();
    });

    appWindow.listen("tauri://blur", ({ event, payload }) => {
      console.log("blur");
      translation = "";
      appWindow.hide();
    });
  });
</script>

<main>
  <div class="translation">
    {@html translation}
  </div>
</main>

<style>
  .translation {
    background: #000;
    color: #fff;
    max-width: 400px;
    padding: 10px;
    border-radius: 5px;
    font-size: 16px;
    line-height: 20px;
    opacity: 0.9;
    text-align: left;
  }
  .translation:hover {
    opacity: 1;
  }
</style>
