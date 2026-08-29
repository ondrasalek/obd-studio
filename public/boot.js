(function () {
  "use strict";

  var bootErrors = [];

  function describeError(event) {
    if (event && event.reason) {
      var rejectionMessage = event.reason.message || String(event.reason);
      return event.reason.stack
        ? rejectionMessage + "\n" + event.reason.stack
        : rejectionMessage;
    }

    if (event && event.error) {
      var errorMessage = event.message || event.error.message || String(event.error);
      return event.error.stack ? errorMessage + "\n" + event.error.stack : errorMessage;
    }

    if (event && event.target && event.target !== window) {
      var source = event.target.src || event.target.href || event.target.tagName;
      return "Nepodařilo se načíst frontendový asset: " + source;
    }

    return (event && event.message) || "Neznámá chyba při startu frontendu.";
  }

  function rememberError(event) {
    bootErrors.push(describeError(event));
  }

  window.addEventListener("error", rememberError, true);
  window.addEventListener("unhandledrejection", rememberError);

  window.setTimeout(function () {
    var root = document.getElementById("root");
    if (!root || root.childElementCount > 0) {
      return;
    }

    var panel = document.createElement("main");
    panel.style.cssText =
      "box-sizing:border-box;min-height:100vh;padding:32px;background:#060a0e;" +
      "color:#e5eef5;font:14px -apple-system,BlinkMacSystemFont,sans-serif;";

    var title = document.createElement("h1");
    title.textContent = "OBD Studio se nepodařilo spustit";
    title.style.cssText = "margin:0 0 12px;font-size:20px;";

    var description = document.createElement("p");
    description.textContent =
      "Frontend skončil před vykreslením. Níže je diagnostika této instalace:";
    description.style.cssText = "margin:0 0 16px;color:#9fb0bd;";

    var details = document.createElement("pre");
    details.textContent = bootErrors.length
      ? bootErrors.join("\n\n")
      : "Nebyla zachycena JavaScript chyba. Pravděpodobně se nenačetl hlavní modul.";
    details.style.cssText =
      "margin:0;white-space:pre-wrap;overflow-wrap:anywhere;border:1px solid #263641;" +
      "border-radius:8px;padding:16px;background:#0c1319;color:#ffb4ab;font:12px ui-monospace,monospace;";

    panel.appendChild(title);
    panel.appendChild(description);
    panel.appendChild(details);
    root.appendChild(panel);
  }, 1500);
})();
