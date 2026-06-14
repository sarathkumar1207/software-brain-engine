document.documentElement.dataset.ready = "true";

for (const button of document.querySelectorAll("[data-copy]")) {
  button.addEventListener("click", async () => {
    const original = button.textContent;
    try {
      await navigator.clipboard.writeText(button.dataset.copy);
      button.textContent = "Copied";
    } catch {
      button.textContent = "Select command";
    }
    window.setTimeout(() => { button.textContent = original; }, 1600);
  });
}
