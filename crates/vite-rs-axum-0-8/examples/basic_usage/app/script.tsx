import React, { lazy } from "react";
import ReactDOM from "react-dom/client";

(async () => {
  const App = await lazy(() => import("./App"));

  ReactDOM.createRoot(document.body).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
})();
