import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { MantineProvider, createTheme } from "@mantine/core";
import { Notifications } from "@mantine/notifications";
import { AppProvider } from "./context/AppContext";
import App from "./App";
import "leaflet/dist/leaflet.css";
import "@mantine/core/styles.css";
import "@mantine/notifications/styles.css";
import "./index.css";

const theme = createTheme({
  primaryColor: "michelin",
  colors: {
    michelin: [
      "#fce8ea",
      "#f4c0c6",
      "#e897a2",
      "#dc6e7e",
      "#d0455a",
      "#c41c36",
      "#AB152E",
      "#921127",
      "#790e20",
      "#600b19",
    ],
  },
  fontFamily: "inherit",
  defaultRadius: "md",
  components: {
    Button: {
      defaultProps: { radius: "xl" },
    },
    TextInput: {
      defaultProps: { radius: "xl" },
    },
    Card: {
      defaultProps: { radius: "xl" },
    },
    Badge: {
      defaultProps: { radius: "xl" },
    },
  },
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <MantineProvider theme={theme}>
    <Notifications />
    <BrowserRouter>
      <AppProvider>
        <App />
      </AppProvider>
    </BrowserRouter>
  </MantineProvider>
);
