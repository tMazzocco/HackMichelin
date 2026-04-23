import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { MantineProvider } from "@mantine/core";
import { Notifications } from "@mantine/notifications";
import { AppProvider } from "./context/AppContext";
import App from "./App";
import "leaflet/dist/leaflet.css";
import "@mantine/core/styles.css";
import "@mantine/notifications/styles.css";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <MantineProvider>
    <Notifications />
    <BrowserRouter>
      <AppProvider>
        <App />
      </AppProvider>
    </BrowserRouter>
  </MantineProvider>
);
