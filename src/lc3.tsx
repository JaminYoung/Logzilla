import React from "react";
import ReactDOM from "react-dom/client";
import { LC3ToolKitWindow } from "./components/LC3ToolKit/LC3ToolKitWindow";
import "./styles/index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <LC3ToolKitWindow />
  </React.StrictMode>,
);
