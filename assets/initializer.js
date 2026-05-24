export default function initializer() {
  return {
    onStart: () => {
      document.body.insertAdjacentHTML(
        "afterbegin",
        `
<div id="app-loader">
  <div id="loader-ring"></div>
  <span class="loader-label" id="loader-text">Loading...</span>
  <div id="error-container">
    <div class="error-icon">⚠️</div>
    <div class="error-title" id="error-title">Failed to load application</div>
    <div class="error-message" id="error-message"></div>
  </div>
</div>`,
      );
      console.log("Loading...");
      console.time("trunk-initializer");

      window.__loaderState = { loading: true, error: null };
    },

    onProgress: ({ current, total }) => {
      if (!total) {
        console.log("Loading...", current, "bytes");
      } else {
        const value = Math.round((current / total) * 100);
        console.log("Loading... ", value, "%");
        updateProgress(value);
      }
    },

    onComplete: () => {
      console.log("Loading... done!");
      console.timeEnd("trunk-initializer");
      window.__loaderState.loading = false;
    },

    onSuccess: (_wasm) => {
      console.log("Loading... successful!");
      hideLoader();
    },

    onFailure: (error) => {
      console.error("Loading... failed!", error);
      showLoaderError(error.message || "Failed to load application");
    },
  };
}

function updateProgress(value) {
  const text = document.getElementById("loader-text");
  if (text && window.__loaderState.loading) {
    text.textContent = `Loading... ${value}%`;
  }
}

function hideLoader() {
  const loader = document.getElementById("app-loader");
  if (!window.__loaderState.loading && loader) {
    loader.classList.add("hidden");
    window.__loaderState.loading = false;
  }
}

function showLoaderError(message) {
  const loader = document.getElementById("app-loader");
  const errorContainer = document.getElementById("error-container");
  const errorTitle = document.getElementById("error-title");
  const errorMessage = document.getElementById("error-message");

  if (loader && errorContainer) {
    errorContainer.style.display = "flex";
    errorMessage.textContent = message;
    loader.classList.add("hidden");
    window.__loaderState.loading = false;
    window.__loaderState.error = message;
  }
}
